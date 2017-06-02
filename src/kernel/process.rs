use alloc::arc::Arc;
use arch::process::ArchProcess;
use arch::thread as arch_thread;
use core::intrinsics;
use core::mem;
use core::nonzero::NonZero;
use core::slice;
use deferred::Deferred;
use elf::*;
use kobj::{KObj,KObjRef};
use multiboot::multiboot_module_t;
use phys_mem::{self,PhysicalBitmap};
use prelude::*;
use process;
use ptr::{self,Align,PointerInSlice};
use spin::Mutex;
use syscall::{ErrNum,Handle,Result};
use tar;
use thread;
use virt_mem::VirtualTree;

macro_rules! try_or_none {
    ($e:expr) => ({
        match $e {
            Some(e) => e,
            None => return None,
        }
    })
}

macro_rules! try_or_false {
    ($e:expr) => ({
        match $e {
            Some(e) => e,
            None => return false,
        }
    })
}

pub struct SharedMemBlock(Mutex<Vec<Option<NonZero<usize>>>>);

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
    handles: Vec<Option<Arc<KObj>>>,
    exit_code: Deferred<i32>
}

impl ProcessState {
    fn new(handles: Vec<Option<Arc<KObj>>>) -> Self {
        ProcessState {
            handles: handles,
            exit_code: Deferred::new()
        }
    }

    fn make_handle(&mut self, obj: Arc<KObj>) -> Handle {
        let handle = self.handles.len();
        self.handles.push(Some(obj));
        handle
    }

    fn resolve_handle_ref(&self, handle: Handle) -> Result<Arc<KObj>> {
        self.handles
            .get(handle)
            .map(|ref o| (*o).clone())
            .unwrap_or(None)
            .ok_or(ErrNum::InvalidHandle)
    }

    fn close_handle(&mut self, handle: Handle) -> bool {
        if let Some(r @ &mut Some(_)) = self.handles.get_mut(handle) {
            *r = None;
            true
        } else {
            false
        }
    }

    fn set_deferred(&mut self, d: Deferred<i32>) {
        self.exit_code = d;
    }
}

#[derive(Clone)]
enum Pager {
    Zeroed,
    Physical(usize),
    Shared(KObjRef<SharedMemBlock>)
}

impl Pager {
    fn alloc<AllocPage: Fn() -> Option<usize>>(&self, offset: usize, alloc_page: AllocPage) -> Option<(bool, usize)> {
        let result =
            match self {
                &Pager::Zeroed => (true, try_or_none!(alloc_page())),
                &Pager::Physical(addr) => (false, addr + offset),

                &Pager::Shared(ref pages) => {
                    let mut pages = lock!(pages.0);
                    let index = offset / phys_mem::PAGE_SIZE;
                    if pages.len() <= index {
                        pages.resize(index + 1, None);
                    }

                    match pages[index] {
                        Some(addr) => (false, addr.get()),
                        None => {
                            let addr = try_or_none!(alloc_page());
                            assert!(addr != 0);
                            pages[index] = Some(unsafe { NonZero::new(addr) });
                            (true, addr)
                        }
                    }
                }
            };

        Some(result)
    }
}

#[derive(Clone)]
struct MemBlock {
    user: bool,
    writable: bool,
    pager: Option<Pager>
}

pub struct Process {
    name: String,
    arch: ArchProcess,
    phys: Arc<PhysicalBitmap>,
    user_virt: VirtualTree<MemBlock>,
    kernel_virt: Arc<VirtualTree<MemBlock>>,
    state: Mutex<ProcessState>
}

impl Process {
    fn new(name: String, phys: Arc<PhysicalBitmap>, kernel_virt: Arc<VirtualTree<MemBlock>>, handles: Vec<Option<Arc<KObj>>>) -> Result<Self> {
        let arch = ArchProcess::new(phys.clone())?;
        let user_virt = VirtualTree::new();
        user_virt.reserve(
            unsafe { slice::from_raw_parts_mut(0 as *mut u8, 4096) },
            MemBlock {
                user: false,
                writable: false,
                pager: None
            });

        Ok(Process {
            name: name,
            arch: arch,
            phys: phys,
            user_virt: user_virt,
            kernel_virt: kernel_virt,
            state: Mutex::new(ProcessState::new(handles))
        })
    }

    pub fn for_kernel() -> Result<Self> {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let identity = phys_mem::identity_range();

        let user_plus_identity = unsafe {
            let kernel_end_ptr = identity.as_ptr().offset(identity.len() as isize);
            slice::from_raw_parts_mut(0 as *mut u8, kernel_end_ptr as usize)
        };

        kernel_virt.reserve(
            user_plus_identity,
            MemBlock {
                user: false,
                writable: false,
                pager: None
            });

        Process::new("<kernel>".into(), phys, kernel_virt, Vec::new())
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn spawn<I: IntoIterator<Item=Handle>>(&self, name: String, inherit: I) -> Result<Self> {
        let mut handles = Vec::new();
        let state = lock!(self.state);
        for handle in inherit {
            handles.push(Some(state.resolve_handle_ref(handle)?));
        }

        Process::new(name, self.phys.clone(), self.kernel_virt.clone(), handles)
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

    pub fn make_handle(&self, obj: Arc<KObj>) -> Handle {
        let mut state = lock!(self.state);
        state.make_handle(obj)
    }

    pub fn resolve_handle_ref<'a, T: 'a+?Sized, F: FnOnce(&'a KObj) -> Option<&'a T>>(&self, handle: Handle, f: F) -> Result<KObjRef<T>> {
        let kobj = lock!(self.state).resolve_handle_ref(handle)?;
        KObjRef::new(kobj, f)
    }

    pub fn resolve_handle<T: Clone, F: FnOnce(&KObj) -> Option<T>>(&self, handle: Handle, f: F) -> Result<T> {
        let kobj = lock!(self.state).resolve_handle_ref(handle)?;
        f(&*kobj).ok_or(ErrNum::NotSupported)
    }

    pub fn exit_code(&self) -> Deferred<i32> {
        lock!(self.state).exit_code.clone()
    }

    pub fn set_exit_code(&self, d: Deferred<i32>) {
        lock!(self.state).set_deferred(d)
    }
}

impl KObj for Process {
    fn deferred_i32(&self) -> Option<Deferred<i32>> {
        Some(self.exit_code())
    }

    fn process(&self) -> Option<&Process> {
        Some(self)
    }
}

pub fn spawn<I: IntoIterator<Item=Handle>>(executable: String, inherit: I) -> Result<(Arc<Process>)> {
    let current = thread::current_process();
    let process = Arc::new(current.spawn(executable.clone(), inherit)?);

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

            tar::locate(mod_data, &executable).ok_or(ErrNum::FileNotFound)?
        };

        mem::drop(executable);

        let ehdr = unsafe { *(image_slice.as_ptr() as *const Elf64_Ehdr) };
        assert_eq!([ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3, ELFCLASS64, ELFDATA2LSB, EV_CURRENT], &ehdr.e_ident[0..7]);
        assert_eq!((ET_EXEC, EM_X86_64), (ehdr.e_type, ehdr.e_machine));
        assert!(ehdr.e_entry != 0);

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
            slice[..file_slice.len()].copy_from_slice(file_slice);
            slices.push(slice);
        }

        assert!(slices.iter().any(|slice| slice.contains_ptr(entry)));

        let stack_slice = process::alloc(phys_mem::PAGE_SIZE * 10, true, true).unwrap();
        log!("stack_slice = 0x{:x} bytes @ {:p}", stack_slice.len(), stack_slice.as_ptr());
        Ok((entry, stack_slice))
    };

    let deferred = thread::spawn_remote(process.clone(), move || {
        let result : Result<_> = init_in_new_process();
        match result {
            Ok((entry, stack_slice)) => {
                unsafe { arch_thread::jmp_user_mode(entry, stack_slice.as_mut_ptr().offset(stack_slice.len() as isize)) }
                // TODO: free stack
            },

            Err(num) => {
                thread::exit(-(num as i32));
            }
        }
    });

    process.set_exit_code(deferred);
    Ok(process)
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
            let ptr = process.alloc_inner(base, len, self.user, self.writable, self.pager)?;
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
    let process = try_or_false!(thread::try_current_process());
    let ptr = Align::down(ptr, phys_mem::PAGE_SIZE);
    let identity = phys_mem::identity_range();
    if identity.contains_ptr(ptr) {
        return unsafe { process.arch.map(ptr, phys_mem::virt2phys(ptr), false, true).is_ok() };
    }

    let (slice, block) = try_or_false!(process.user_virt.tag_at(ptr).or_else(|| process.kernel_virt.tag_at(ptr)));
    assert!(slice.as_mut_ptr() <= ptr && ptr < unsafe { slice.as_mut_ptr().offset(slice.len() as isize) });

    let pager = try_or_false!(block.pager);
    let offset = ptr::bytes_between(slice.as_mut_ptr(), ptr);
    let (dirty, addr) = try_or_false!(pager.alloc(offset, || process.phys.alloc_page().ok()));

    unsafe {
        try_or_false!(process.arch.map(ptr, addr, block.user, block.writable).ok());

        if dirty {
            intrinsics::write_bytes(ptr, 0, phys_mem::PAGE_SIZE);
        }
    }

    true
}

pub fn make_handle(obj: Arc<KObj>) -> Handle {
    thread::current_process().make_handle(obj)
}

pub fn resolve_handle_ref<'a, T: 'a+?Sized, F: FnOnce(&'a KObj) -> Option<&'a T>>(handle: Handle, f: F) -> Result<KObjRef<T>> {
    thread::current_process().resolve_handle_ref(handle, f)
}

pub fn resolve_handle<T: Clone, F: FnOnce(&KObj) -> Option<T>>(handle: Handle, f: F) -> Result<T> {
    thread::current_process().resolve_handle(handle, f)
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
                assert!(slice.iter().all(|&b| b == 0));

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

                let d1 = thread::spawn_remote(Arc::new(p.spawn("user_addresses_are_separate(d1)".into(), vec![]).unwrap()), || unsafe {
                    let slice = alloc_at(0x1000 as *mut _, 0x1000, true, true).unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 123);
                    0
                });

                let d2 = thread::spawn_remote(Arc::new(p.spawn("user_addresses_are_separate(d2)".into(), vec![]).unwrap()), || unsafe {
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

                let d1 = thread::spawn_remote(Arc::new(p.spawn("can_share_memory(d1)".into(), vec![]).unwrap()), || unsafe {
                    let slice = Allocation::shared(0x1000, shared1).base(0x1000_0000 as *mut u8).user(true).writable(true).allocate().unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 123);
                    0
                });

                d1.get();

                let d2 = thread::spawn_remote(Arc::new(p.spawn("can_share_memory(d2)".into(), vec![]).unwrap()), || unsafe {
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
