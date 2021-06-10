use crate::arch::process::ArchProcess;
use crate::arch::thread as arch_thread;
use crate::deferred::Deferred;
use crate::elf::*;
use crate::kobj::{KObj, KObjRef};
use crate::phys_mem::{self, PhysicalBitmap};
use crate::prelude::*;
use crate::process;
use crate::ptr::{self, Align, PointerInSlice};
use crate::spin::Mutex;
use crate::tar;
use crate::thread;
use crate::virt_mem::VirtualTree;
use alloc::sync::Arc;
use core::intrinsics;
use core::mem;
use core::num::NonZeroUsize;
use core::slice;
use syscall::{ErrNum, Handle, Result};

macro_rules! try_or_none {
    ($e:expr) => {{
        match $e {
            Some(e) => e,
            None => {
                log!("page fault is not handled because: {}", stringify!($e));
                return None;
            }
        }
    }};
}

macro_rules! try_or_false {
    ($e:expr) => {{
        match $e {
            Some(e) => e,
            None => {
                log!("page fault is not handled because: {}", stringify!($e));
                return false;
            }
        }
    }};
}

pub struct SharedMemBlock {
    phys: Arc<PhysicalBitmap>,
    pages: Mutex<Vec<Option<NonZeroUsize>>>,
}

impl KObj for SharedMemBlock {
    fn shared_mem_block(&self) -> Option<&SharedMemBlock> {
        Some(&self)
    }
}

impl SharedMemBlock {
    fn alloc(&self, offset: usize) -> Option<(bool, usize)> {
        let mut pages = lock!(self.pages);
        let index = offset / phys_mem::PAGE_SIZE;
        if pages.len() <= index {
            pages.resize(index + 1, None);
        }

        match pages[index] {
            Some(addr) => Some((false, addr.get())),
            None => {
                let addr = try_or_none!(self.phys.alloc_page().ok());
                assert_ne!(addr, 0);
                pages[index] = Some(unsafe { NonZeroUsize::new_unchecked(addr) });
                Some((true, addr))
            }
        }
    }
}

impl Drop for SharedMemBlock {
    fn drop(&mut self) {
        for page in self.pages.get_mut().drain(..) {
            if let Some(addr) = page {
                self.phys.free_page(addr.get())
            }
        }
    }
}

struct ProcessState {
    handles: Vec<Option<Arc<dyn KObj>>>,
    exit_code: Deferred<i32>,
    tls: Option<(usize, &'static [u8])>,
}

impl ProcessState {
    fn new(handles: Vec<Option<Arc<dyn KObj>>>) -> Self {
        ProcessState {
            handles,
            exit_code: Deferred::new(),
            tls: None,
        }
    }

    fn make_handle(&mut self, obj: Arc<dyn KObj>) -> Handle {
        let handle = self.handles.len();
        self.handles.push(Some(obj));
        handle
    }

    fn resolve_handle_obj(&self, handle: Handle) -> Result<Arc<dyn KObj>> {
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

    fn set_tls(&mut self, len: usize, data: &'static [u8]) {
        self.tls = Some((len, data));
    }
}

enum Pager {
    Zeroed(SharedMemBlock),
    Physical(usize),
    Shared(KObjRef<SharedMemBlock>),
}

impl Pager {
    fn alloc(&self, offset: usize) -> Option<(bool, usize)> {
        match self {
            Self::Zeroed(block) => block.alloc(offset),
            Self::Physical(addr) => Some((false, *addr + offset)),
            Self::Shared(block) => block.alloc(offset),
        }
    }
}

#[derive(Clone)]
struct MemBlock {
    user: bool,
    writable: bool,
    pager: Option<Arc<Pager>>,
}

pub struct Process {
    name: String,
    arch: ArchProcess,
    phys: Arc<PhysicalBitmap>,
    user_virt: VirtualTree<MemBlock>,
    kernel_virt: Arc<VirtualTree<MemBlock>>,
    state: Mutex<ProcessState>,
}

impl Process {
    fn new(
        name: String,
        phys: Arc<PhysicalBitmap>,
        kernel_virt: Arc<VirtualTree<MemBlock>>,
        handles: Vec<Option<Arc<dyn KObj>>>,
    ) -> Result<Self> {
        let arch = ArchProcess::new(phys.clone())?;
        let user_virt = VirtualTree::new();
        user_virt.reserve(
            unsafe { slice::from_raw_parts_mut(0 as *mut u8, 4096) },
            MemBlock {
                user: false,
                writable: false,
                pager: None,
            },
        );

        Ok(Process {
            name,
            arch,
            phys,
            user_virt,
            kernel_virt,
            state: Mutex::new(ProcessState::new(handles)),
        })
    }

    pub fn for_kernel() -> Result<Self> {
        let phys = Arc::new(PhysicalBitmap::machine());
        let kernel_virt = Arc::new(VirtualTree::new());
        let identity = phys_mem::identity_range();

        let user_plus_identity = unsafe {
            let kernel_end_ptr = identity.as_ptr().offset(identity.len() as isize);
            slice::from_raw_parts_mut(
                phys_mem::PAGE_SIZE as *mut u8,
                kernel_end_ptr as usize - phys_mem::PAGE_SIZE,
            )
        };

        kernel_virt.reserve(
            user_plus_identity,
            MemBlock {
                user: false,
                writable: false,
                pager: None,
            },
        );

        Process::new("<kernel>".into(), phys, kernel_virt, Vec::new())
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn spawn(&self, name: String, handles: Vec<Option<Arc<dyn KObj>>>) -> Result<Self> {
        Process::new(name, self.phys.clone(), self.kernel_virt.clone(), handles)
    }

    pub unsafe fn switch(&self) {
        self.arch.switch();
    }

    unsafe fn alloc_inner(
        &self,
        ptr_opt: Option<*mut u8>,
        len: usize,
        user: bool,
        writable: bool,
        pager: Pager,
    ) -> Result<*mut u8> {
        let virt = if user { &self.user_virt } else { &*self.kernel_virt };

        let block = MemBlock {
            user,
            writable,
            pager: Some(Arc::new(pager)),
        };

        match ptr_opt {
            Some(ptr) => {
                if virt.reserve(slice::from_raw_parts_mut(ptr, len), block) {
                    Ok(ptr)
                } else {
                    log!("can't reserve {} bytes at {:p}", len, ptr);
                    Err(ErrNum::InvalidArgument)
                }
            }

            None => virt.alloc(len, block).map(|slice| slice.as_mut_ptr()),
        }
    }

    pub fn make_handle(&self, obj: Arc<dyn KObj>) -> Handle {
        let mut state = lock!(self.state);
        state.make_handle(obj)
    }

    pub fn resolve_handle_ref<'a, T: 'a + ?Sized, F: FnOnce(&'a dyn KObj) -> Option<&'a T>>(
        &self,
        handle: Handle,
        f: F,
    ) -> Result<KObjRef<T>> {
        let kobj = lock!(self.state).resolve_handle_obj(handle)?;
        KObjRef::new(kobj, f)
    }

    pub fn resolve_handle<T: Clone, F: FnOnce(&dyn KObj) -> Option<T>>(&self, handle: Handle, f: F) -> Result<T> {
        let kobj = lock!(self.state).resolve_handle_obj(handle)?;
        f(&*kobj).ok_or(ErrNum::NotSupported)
    }

    pub fn resolve_handle_obj(&self, handle: Handle) -> Result<Arc<dyn KObj>> {
        lock!(self.state).resolve_handle_obj(handle)
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

pub fn set_tls(len: usize, data: &'static [u8]) {
    let process = thread::current_process();
    let mut state = lock!(process.state);
    state.set_tls(len, data);
}

pub fn alloc_tls() -> Option<&'static mut [u8]> {
    let process = thread::current_process();
    let state = lock!(process.state);
    state.tls.map(|(len, data)| {
        let tls_slice = process::alloc::<u8>(len, true, true).unwrap();
        tls_slice[..data.len()].copy_from_slice(data);
        tls_slice
    })
}

#[cfg(not(target_arch = "arm"))]
pub fn spawn(executable: String, handles: Vec<Option<Arc<dyn KObj>>>) -> Result<Arc<Process>> {
    let current = thread::current_process();
    let process = Arc::new(current.spawn(executable.clone(), handles)?);

    let init_in_new_process = move || -> Result<_> {
        let image_slice = unsafe {
            use crate::arch::multiboot::multiboot_module_t;
            use crate::arch::phys_mem::multiboot_info;

            let info = multiboot_info();

            let mods: &[multiboot_module_t] =
                slice::from_raw_parts(phys_mem::phys2virt(info.mods_addr as usize), info.mods_count as usize);

            assert_eq!(1, mods.len());

            let mod_data: &[u8] = slice::from_raw_parts(
                phys_mem::phys2virt(mods[0].mod_start as usize),
                (mods[0].mod_end - mods[0].mod_start) as usize,
            );

            tar::locate(mod_data, &executable).ok_or(ErrNum::FileNotFound)?
        };

        mem::drop(executable);

        let ehdr = unsafe { &*(image_slice.as_ptr() as *const Elf64_Ehdr) };
        assert_eq!(
            [ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3, ELFCLASS64, ELFDATA2LSB, EV_CURRENT],
            &ehdr.e_ident[0..7]
        );
        assert_eq!((ET_EXEC, EM_X86_64), (ehdr.e_type, ehdr.e_machine));
        assert_ne!(ehdr.e_entry, 0);

        let entry = ehdr.e_entry as *const u8;
        let mut slices = Vec::new();
        let mut tls = None;
        for i in 0..ehdr.e_phnum {
            let phdr_offset = ehdr.e_phoff as isize + (i as isize) * (ehdr.e_phentsize as isize);
            let phdr = unsafe { &*(image_slice.as_ptr().offset(phdr_offset) as *const Elf64_Phdr) };
            match phdr.p_type {
                PT_LOAD => {
                    assert!(phdr.p_memsz >= phdr.p_filesz);

                    let slice = unsafe {
                        process::alloc_at::<u8>(phdr.p_vaddr as *mut u8, phdr.p_memsz as usize, true, true).unwrap()
                    };

                    let file_slice = &image_slice[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize];
                    slice[..file_slice.len()].copy_from_slice(file_slice);
                    slices.push(slice);
                }

                PT_TLS => {
                    assert!(phdr.p_memsz >= phdr.p_filesz);
                    assert!(tls.is_none(), "segment {}: didn't expect another TLS segment", i);

                    let slice = process::alloc::<u8>(phdr.p_filesz as usize, true, false).unwrap();
                    let file_slice = &image_slice[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize];
                    slice.copy_from_slice(file_slice);
                    tls = Some((phdr.p_memsz as usize, slice as &[u8]));
                }

                PT_GNU_STACK => {
                    assert_eq!(phdr.p_memsz, 0);
                }

                _ => {
                    panic!("segment {}: don't know how to handle type {}", i, phdr.p_type);
                }
            }
        }

        assert!(slices.iter().any(|slice| slice.contains_ptr(entry)));

        let stack_slice = process::alloc::<u8>(phys_mem::PAGE_SIZE * 10, true, true).unwrap();

        if let Some((len, data)) = tls {
            process::set_tls(len, data);
        }

        Ok((entry, stack_slice))
    };

    let deferred = thread::spawn_remote(process.clone(), move || {
        let (rip, stack_slice) = match init_in_new_process() {
            Ok(tuple) => tuple,
            Err(num) => thread::exit(-(num as i32)),
        };

        if let Some(tls) = process::alloc_tls() {
            thread::set_tls(tls);
        }

        unsafe {
            let rsp = stack_slice.as_mut_ptr().offset(stack_slice.len() as isize);
            arch_thread::jmp_user_mode(rip, rsp, 0)
        }
        // TODO: free stack
    });

    process.set_exit_code(deferred);
    Ok(process)
}

struct Allocation<T> {
    base: Option<*mut T>,
    user: bool,
    writable: bool,
}

impl<T> Allocation<T> {
    fn zeroed(self, len: usize) -> Result<&'static mut [T]> {
        let process = thread::current_process();
        let base = self.base.map(|ptr| ptr as *mut u8);
        let byte_len = (len * mem::size_of::<T>()).max(phys_mem::PAGE_SIZE);

        let pager = Pager::Zeroed(SharedMemBlock {
            phys: process.phys.clone(),
            pages: Mutex::new(Vec::new()),
        });

        unsafe {
            let ptr = process.alloc_inner(base, byte_len, self.user, self.writable, pager)?;
            Ok(slice::from_raw_parts_mut(ptr as *mut T, len))
        }
    }

    fn phys(self, len: usize, addr: usize) -> Result<&'static mut [T]> {
        let process = thread::current_process();
        let base = self.base.map(|ptr| ptr as *mut u8);
        let byte_len = (len * mem::size_of::<T>()).max(phys_mem::PAGE_SIZE);
        let pager = Pager::Physical(addr);

        unsafe {
            let ptr = process.alloc_inner(base, byte_len, self.user, self.writable, pager)?;
            Ok(slice::from_raw_parts_mut(ptr as *mut T, len))
        }
    }

    fn shared(self, len: usize, block: KObjRef<SharedMemBlock>) -> Result<&'static mut [T]> {
        let process = thread::current_process();
        let base = self.base.map(|ptr| ptr as *mut u8);
        let byte_len = (len * mem::size_of::<T>()).max(phys_mem::PAGE_SIZE);
        let pager = Pager::Shared(block);
        unsafe {
            let ptr = process.alloc_inner(base, byte_len, self.user, self.writable, pager)?;
            Ok(slice::from_raw_parts_mut(ptr as *mut T, len))
        }
    }
}

pub fn alloc<T>(len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    let a = Allocation {
        base: None,
        user,
        writable,
    };
    a.zeroed(len)
}

pub unsafe fn alloc_at<T>(base: *mut T, len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    let a = Allocation {
        user,
        writable,
        base: Some(base),
    };
    a.zeroed(len)
}

pub unsafe fn map_phys<T>(addr: usize, len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    let a = Allocation {
        base: None,
        user,
        writable,
    };
    a.phys(len, addr)
}

pub fn map_shared<T>(
    block: KObjRef<SharedMemBlock>,
    len: usize,
    user: bool,
    writable: bool,
) -> Result<&'static mut [T]> {
    let a = Allocation {
        user,
        writable,
        base: None,
    };
    a.shared(len, block)
}

pub fn create_shared_mem() -> SharedMemBlock {
    let process = thread::current_process();
    SharedMemBlock {
        phys: process.phys.clone(),
        pages: Mutex::new(Vec::new()),
    }
}

pub fn free(ptr: *mut u8) -> bool {
    let process = thread::current_process();
    let ptr = Align::down(ptr, phys_mem::PAGE_SIZE);
    let (len, _) = if let Some(tuple) = process.user_virt.free(ptr) {
        tuple
    } else {
        return false;
    };

    assert!(Align::is_aligned(len, phys_mem::PAGE_SIZE));

    for offset in (0..len).step_by(phys_mem::PAGE_SIZE) {
        unsafe {
            process
                .arch
                .map(ptr.offset(offset as isize), None, false, false)
                .unwrap()
        }
    }

    true
}

pub fn resolve_page_fault(ptr: *mut u8) -> bool {
    let process = try_or_false!(thread::try_current_process());
    let ptr = Align::down(ptr, phys_mem::PAGE_SIZE);
    let identity = phys_mem::identity_range();
    assert!(!identity.contains_ptr(ptr));

    let (slice, block) = try_or_false!(process
        .user_virt
        .tag_at(ptr)
        .or_else(|| process.kernel_virt.tag_at(ptr)));
    assert!(slice.contains_ptr(ptr));

    let pager = try_or_false!(block.pager);
    let offset = ptr::bytes_between(slice.as_mut_ptr(), ptr);
    let (dirty, addr) = try_or_false!(pager.alloc(offset));

    unsafe {
        if dirty {
            try_or_false!(process.arch.map(ptr, Some(addr), block.user, true).ok());
            intrinsics::write_bytes(ptr, 0, phys_mem::PAGE_SIZE);

            if !block.writable {
                try_or_false!(process.arch.map(ptr, Some(addr), block.user, false).ok());
            }
        } else {
            try_or_false!(process.arch.map(ptr, Some(addr), block.user, block.writable).ok());
        }
    }

    true
}

pub fn make_handle(obj: Arc<dyn KObj>) -> Handle {
    thread::current_process().make_handle(obj)
}

pub fn resolve_handle_ref<'a, T: 'a + ?Sized, F: FnOnce(&'a dyn KObj) -> Option<&'a T>>(
    handle: Handle,
    f: F,
) -> Result<KObjRef<T>> {
    thread::current_process().resolve_handle_ref(handle, f)
}

pub fn resolve_handle<T: Clone, F: FnOnce(&dyn KObj) -> Option<T>>(handle: Handle, f: F) -> Result<T> {
    thread::current_process().resolve_handle(handle, f)
}

pub fn resolve_handle_obj(handle: Handle) -> Result<Arc<dyn KObj>> {
    thread::current_process().resolve_handle_obj(handle)
}

pub fn close_handle(handle: Handle) -> bool {
    let process = thread::current_process();
    let mut state = lock!(process.state);
    state.close_handle(handle)
}

#[cfg(feature = "test")]
pub mod test {
    use super::*;
    use crate::thread;
    use alloc::sync::Arc;
    use core::intrinsics;

    test! {
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
                let shared = KObjRef::new(Arc::new(process::create_shared_mem()), |kobj| kobj.shared_mem_block()).unwrap();
                let p = thread::current_process();
                let shared1 = shared.clone();
                let shared2 = shared.clone();

                let d1 = thread::spawn_remote(Arc::new(p.spawn("can_share_memory(d1)".into(), vec![]).unwrap()), || unsafe {
                    let a = Allocation {
                        base: Some(0x1000_0000 as *mut u8),
                        user: true,
                        writable: true
                    };

                    let slice = a.shared(0x1000, shared1).unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 123);
                    0
                });

                d1.get();

                let d2 = thread::spawn_remote(Arc::new(p.spawn("can_share_memory(d2)".into(), vec![]).unwrap()), || unsafe {
                    let a = Allocation {
                        base: Some(0x2000_0000 as *mut u8),
                        user: true,
                        writable: false
                    };

                    let slice = a.shared(0x1000, shared2).unwrap();
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
