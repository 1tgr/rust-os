use alloc::arc::Arc;
use arch::process::ArchProcess;
use arch::thread as arch_thread;
use core::mem;
use core::nonzero::NonZero;
use core::ops::Deref;
use core::slice::{self,bytes};
use deferred::Deferred;
use elf::*;
use io::{AsyncRead,Read,Write};
use multiboot::multiboot_module_t;
use mutex::Mutex;
use phys_mem::{self,PhysicalBitmap};
use prelude::*;
use process;
use ptr::{self,Align};
use syscall::{ErrNum,Handle,Result};
use tar;
use thread;
use virt_mem::VirtualTree;

macro_rules! try_option {
    ($e:expr) => ({
        match $e {
            Some(e) => e,
            None => return false,
        }
    })
}

pub struct SharedMemBlock(Mutex<Vec<Option<NonZero<usize>>>>);

pub trait KObj {
    fn async_read(&self) -> Option<&AsyncRead> { None }
    fn read(&self) -> Option<&Read> { None }
    fn write(&self) -> Option<&Write> { None }
    fn deferred_i32(&self) -> Option<&Deferred<i32>> { None }
    fn shared_mem_block(&self) -> Option<&SharedMemBlock> { None }
}

pub struct KObjRef<T: ?Sized> {
    kobj: Arc<KObj>,
    ptr: NonZero<*const T>
}

impl<'a, T: ?Sized+'a> KObjRef<T> {
    pub fn new<F: FnOnce(&'a KObj) -> Option<&'a T>>(kobj: Arc<KObj>, f: F) -> Result<Self> {
        let ptr = {
            let kobj: &KObj = &*kobj;
            let kobj: &'a KObj = unsafe { mem::transmute(kobj) };
            match f(kobj) {
                Some(r) => r as *const T,
                None => { return Err(ErrNum::NotSupported) }
            }
        };

        Ok(KObjRef { kobj: kobj, ptr: unsafe { NonZero::new(ptr) } })
    }
}

impl<T: ?Sized> Clone for KObjRef<T> {
    fn clone(&self) -> Self {
        KObjRef { kobj: self.kobj.clone(), ptr: self.ptr }
    }
}

impl<T: ?Sized> Deref for KObjRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let ptr: *const T = *self.ptr;
        unsafe { &*ptr }
    }
}

impl SharedMemBlock {
    pub fn new() -> Self {
        SharedMemBlock(Mutex::new(Vec::new()))
    }
}

impl KObj for SharedMemBlock {
    fn shared_mem_block(&self) -> Option<&SharedMemBlock> {
        Some(&self)
    }
}

struct ProcessState {
    handles: Vec<Option<Arc<KObj>>>
}

impl ProcessState {
    fn new() -> Self {
        ProcessState {
            handles: Vec::new()
        }
    }

    fn make_handle(&mut self, obj: Arc<KObj>) -> Handle {
        let handle = self.handles.len();
        self.handles.push(Some(obj));
        handle
    }

    fn resolve_handle(&self, handle: Handle) -> Option<Arc<KObj>> {
        self.handles.get(handle).map(|ref o| (*o).clone()).unwrap_or(None)
    }

    fn close_handle(&mut self, handle: Handle) -> bool {
        if let Some(r @ &mut Some(_)) = self.handles.get_mut(handle) {
            *r = None;
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
enum Pager {
    Zeroed,
    Physical(usize),
    Shared(KObjRef<SharedMemBlock>)
}

impl Pager {
    fn alloc<AllocPage: Fn() -> Result<usize>>(&self, offset: usize, alloc_page: AllocPage) -> Option<usize> {
        match self {
            &Pager::Zeroed => alloc_page().ok(),
            &Pager::Physical(addr) => Some(addr + offset),

            &Pager::Shared(ref pages) => {
                let mut pages = lock!(pages.0);
                let index = offset / phys_mem::PAGE_SIZE;
                if pages.len() <= index {
                    pages.resize(index + 1, None);
                }

                match pages[index] {
                    Some(addr) => Some(*addr),
                    None => {
                        alloc_page().ok().map(|addr| {
                            assert!(addr != 0);
                            pages[index] = Some(unsafe { NonZero::new(addr) });
                            addr
                        })
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct MemBlock {
    user: bool,
    writable: bool,
    pager: Option<Pager>
}

pub struct Process {
    arch: ArchProcess,
    phys: Arc<PhysicalBitmap>,
    user_virt: VirtualTree<MemBlock>,
    kernel_virt: Arc<VirtualTree<MemBlock>>,
    state: Mutex<ProcessState>
}

impl Process {
    fn new(phys: Arc<PhysicalBitmap>, kernel_virt: Arc<VirtualTree<MemBlock>>) -> Result<Self> {
        let arch = try!(ArchProcess::new(phys.clone()));
        let user_virt = VirtualTree::new();
        user_virt.reserve(
            unsafe { slice::from_raw_parts_mut(0 as *mut u8, 4096) },
            MemBlock {
                user: false,
                writable: false,
                pager: None
            });

        Ok(Process {
            arch: arch,
            phys: phys,
            user_virt: user_virt,
            kernel_virt: kernel_virt,
            state: Mutex::new(ProcessState::new())
        })
    }

    pub fn for_kernel() -> Result<Self> {
        extern {
            static kernel_end: u8;
        }

        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let two_meg = 2 * 1024 * 1024;
        let kernel_end_ptr = Align::up(&kernel_end as *const u8, 4 * two_meg);
        let identity = unsafe { slice::from_raw_parts_mut(0 as *mut u8, kernel_end_ptr as usize) };
        kernel_virt.reserve(
            identity,
            MemBlock {
                user: false,
                writable: false,
                pager: None
            });

        Process::new(phys, kernel_virt)
    }

    pub fn spawn(&self) -> Result<Self> {
        Process::new(self.phys.clone(), self.kernel_virt.clone())
    }

    pub unsafe fn switch(&self) {
        self.arch.switch();
    }

    unsafe fn alloc_inner(&self, ptr_opt: Option<*mut u8>, len: usize, user: bool, writable: bool, pager: Pager) -> Result<*mut u8> {
        let virt = if user { &self.user_virt } else { &*self.kernel_virt };

        let block =
            MemBlock {
                user: user,
                writable: writable,
                pager: Some(pager)
            };

        match ptr_opt {
            Some(ptr) => {
                if virt.reserve(slice::from_raw_parts_mut(ptr, len), block) {
                    Ok(ptr)
                } else {
                    log!("can't reserve {} bytes at {:p}", len, ptr);
                    Err(ErrNum::InvalidArgument)
                }
            },

            None => virt.alloc(len, block).map(|slice| slice.as_mut_ptr())
        }
    }
}

pub fn spawn(executable: String) -> Result<(Arc<Process>, Deferred<i32>)> {
    let process = Arc::new(try!(thread::current_process().spawn()));

    let init_in_new_process = move || {
        let image_slice = unsafe {
            let info = phys_mem::multiboot_info();

            let mods: &[multiboot_module_t] = slice::from_raw_parts(
                phys_mem::phys2virt(info.mods_addr as usize),
                info.mods_count as usize);

            assert_eq!(1, mods.len());

            let mod_data: &[u8] = slice::from_raw_parts(
                phys_mem::phys2virt(mods[0].mod_start as usize),
                (mods[0].mod_end - mods[0].mod_start) as usize);

            try!(tar::locate(mod_data, &executable).ok_or(ErrNum::FileNotFound))
        };

        mem::drop(executable);

        let ehdr = unsafe { *(image_slice.as_ptr() as *const Elf64_Ehdr) };
        assert_eq!([ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3, ELFCLASS64, ELFDATA2LSB, EV_CURRENT], &ehdr.e_ident[0..7]);
        assert_eq!((ET_EXEC, EM_X86_64), (ehdr.e_type, ehdr.e_machine));

        let entry = ehdr.e_entry as *const u8;
        log!("entry point is {:p}", entry);

        let mut slices = Vec::new();
        for i in 0..ehdr.e_phnum {
            let phdr_offset = ehdr.e_phoff as isize + (i as isize) * (ehdr.e_phentsize as isize);
            let phdr = unsafe { *(image_slice.as_ptr().offset(phdr_offset) as *const Elf64_Phdr) };
            if phdr.p_type != PT_LOAD {
                continue;
            }

            log!("segment {}: {:x} bytes @ {:p} (file: {:x} bytes @ {:x})", i, phdr.p_memsz, phdr.p_vaddr as *mut u8, phdr.p_filesz, phdr.p_offset);
            assert!(phdr.p_memsz >= phdr.p_filesz);
            let slice = unsafe { process::alloc_at::<u8>(phdr.p_vaddr as *mut u8, phdr.p_memsz as usize, true, true).unwrap() };
            let file_slice = &image_slice[phdr.p_offset as usize .. (phdr.p_offset + phdr.p_filesz) as usize];
            bytes::copy_memory(file_slice, slice);
            slices.push(slice);
        }

        assert!(slices.iter().any(|ref slice| {
            let slice_end = unsafe { slice.as_ptr().offset(slice.len() as isize) };
            entry >= slice.as_ptr() && entry < slice_end
        }));

        let stack_slice = process::alloc(phys_mem::PAGE_SIZE * 10, true, true).unwrap();
        log!("stack_slice = 0x{:x} bytes @ {:p}", stack_slice.len(), stack_slice.as_ptr());
        Ok((entry, stack_slice))
    };

    let deferred = thread::spawn_remote(process.clone(), move || {
        match init_in_new_process() {
            Ok((entry, stack_slice)) => {
                unsafe { arch_thread::jmp_user_mode(entry, stack_slice.as_mut_ptr().offset(stack_slice.len() as isize)) }
                // TODO: free stack
            },

            Err(num) => {
                thread::exit(-(num as i32));
            }
        }
    });

    Ok((process.clone(), deferred))
}

pub struct Allocation<T> {
    len: usize,
    base: Option<*mut T>,
    user: bool,
    writable: bool,
    pager: Pager
}

impl<T> Allocation<T> {
    fn new(len: usize, pager: Pager) -> Self {
        Allocation {
            base: None,
            len: len,
            user: false,
            writable: false,
            pager: pager
        }
    }

    pub fn zeroed(len: usize) -> Self {
        Allocation::new(len, Pager::Zeroed)
    }

    pub fn phys(len: usize, addr: usize) -> Self {
        Allocation::new(len, Pager::Physical(addr))
    }

    pub fn shared(len: usize, shared: KObjRef<SharedMemBlock>) -> Self {
        Allocation::new(len, Pager::Shared(shared))
    }

    pub fn user(mut self, user: bool) -> Self {
        self.user = user;
        self
    }

    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    pub fn base(mut self, base: *mut T) -> Self {
        self.base = Some(base);
        self
    }

    pub fn allocate(self) -> Result<&'static mut [T]> {
        let process = thread::current_process();
        let base = self.base.map(|ptr| ptr as *mut u8);
        let len = self.len * mem::size_of::<T>();
        unsafe {
            let ptr = try!(process.alloc_inner(base, len, self.user, self.writable, self.pager));
            Ok(slice::from_raw_parts_mut(ptr as *mut T, self.len))
        }
    }
}

pub fn alloc<T>(len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    Allocation::zeroed(len).user(user).writable(writable).allocate()
}

pub unsafe fn alloc_at<T>(base: *mut T, len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    Allocation::zeroed(len).user(user).writable(writable).base(base).allocate()
}

pub unsafe fn map_phys<T>(addr: usize, len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    Allocation::phys(len, addr).user(user).writable(writable).allocate()
}

pub fn map_shared<T>(block: KObjRef<SharedMemBlock>, len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    Allocation::shared(len, block).user(user).writable(writable).allocate()
}

pub fn free(p: *mut u8) -> bool {
    thread::current_process().user_virt.free(p)
}

pub fn resolve_page_fault(ptr: *mut u8) -> bool {
    let process = thread::current_process();
    let ptr = Align::down(ptr, phys_mem::PAGE_SIZE);
    let (slice, block) = try_option!(process.user_virt.tag_at(ptr).or_else(|| process.kernel_virt.tag_at(ptr)));
    assert!(slice.as_mut_ptr() <= ptr && ptr < unsafe { slice.as_mut_ptr().offset(slice.len() as isize) });

    let pager = try_option!(block.pager);
    let offset = ptr::bytes_between(slice.as_mut_ptr(), ptr);
    let addr = try_option!(pager.alloc(offset, || process.phys.alloc_page()));

    unsafe {
        process.arch.map(ptr, addr, block.user, block.writable).is_ok()
    }
}

pub fn make_handle(obj: Arc<KObj>) -> Handle {
    let process = thread::current_process();
    let mut state = lock!(process.state);
    state.make_handle(obj)
}

pub fn resolve_handle<'a, T: 'a+?Sized, F: FnOnce(&'a KObj) -> Option<&'a T>>(handle: Handle, f: F) -> Result<KObjRef<T>> {
    let process = thread::current_process();
    let kobj = try!(lock!(process.state).resolve_handle(handle).ok_or(ErrNum::InvalidHandle));
    KObjRef::new(kobj, f)
}

pub fn close_handle(handle: Handle) -> bool {
    let process = thread::current_process();
    let mut state = lock!(process.state);
    state.close_handle(handle)
}

#[cfg(feature = "test")]
pub mod test {
    use alloc::arc::Arc;
    use core::intrinsics;
    use super::*;
    use thread;

    test!{
        fn can_alloc() {
            thread::with_scheduler(|| {
                let len = 4096;
                let slice = alloc::<u16>(len, false, true).unwrap();
                let sentinel = 0xaa55;
                for i in 0..len {
                    let ptr = &mut slice[i] as *mut u16;
                    unsafe {
                        intrinsics::volatile_store(ptr, sentinel);
                        assert_eq!(sentinel, intrinsics::volatile_load(ptr));
                    }
                }
            });
        }

        fn user_addresses_are_separate() {
            thread::with_scheduler(|| {
                let p = thread::current_process();

                let d1 = thread::spawn_remote(Arc::new(p.spawn().unwrap()), || unsafe {
                    let slice = alloc_at(0x1000 as *mut _, 0x1000, true, true).unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 123);
                    0
                });

                let d2 = thread::spawn_remote(Arc::new(p.spawn().unwrap()), || unsafe {
                    let slice = alloc_at(0x1000 as *mut _, 0x1000, true, true).unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 456);
                    0
                });

                d1.get();
                d2.get();
            });
        }

        fn can_share_memory() {
            thread::with_scheduler(|| {
                let shared = KObjRef::new(Arc::new(SharedMemBlock::new()), |kobj| kobj.shared_mem_block()).unwrap();
                let p = thread::current_process();
                let shared1 = shared.clone();
                let shared2 = shared.clone();

                let d1 = thread::spawn_remote(Arc::new(p.spawn().unwrap()), || unsafe {
                    let slice = Allocation::shared(0x1000, shared1).base(0x1000_0000 as *mut u8).user(true).writable(true).allocate().unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 123);
                    0
                });

                d1.get();

                let d2 = thread::spawn_remote(Arc::new(p.spawn().unwrap()), || unsafe {
                    let slice = Allocation::shared(0x1000, shared2).base(0x2000_0000 as *mut u8).user(true).allocate().unwrap();
                    assert_eq!(123, intrinsics::volatile_load(slice.as_ptr()));
                    0
                });

                d2.get();
            });
        }

        /*fn kernel_addresses_are_shared() {
            thread::with_scheduler(|| {
                let p1 = Arc::new(Process::new(phys.clone(), kernel_virt.clone()).unwrap());
                let p2 = Arc::new(Process::new(phys.clone(), kernel_virt.clone()).unwrap());
                let slice = process::alloc(0x1000, false, true).unwrap();
                unsafe { intrinsics::volatile_store(slice.as_mut_ptr(), 123) };

                let d1 = thread::spawn_remote(p1, || unsafe {
                    assert_eq!(123, intrinsics::volatile_load(slice.as_ptr()));
                });

                let d2 = thread::spawn_remote(p2, || unsafe {
                    assert_eq!(123, intrinsics::volatile_load(slice.as_ptr()));
                });

                d1.get();
                d2.get();
            });
        } */
    }
}
