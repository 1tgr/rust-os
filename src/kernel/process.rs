use alloc::arc::Arc;
use arch::process::ArchProcess;
use arch::thread as arch_thread;
use core::mem;
use core::slice::{self,bytes};
use deferred::Deferred;
use elf::*;
use io::{AsyncRead,Read,Write};
use multiboot::multiboot_module_t;
use mutex::Mutex;
use phys_mem::{self,PhysicalBitmap};
use prelude::*;
use process;
use syscall::{ErrNum,Handle,Result};
use tar;
use thread;
use virt_mem::VirtualTree;

pub trait KObj {
    fn async_read(&self) -> Option<&AsyncRead> { None }
    fn read(&self) -> Option<&Read> { None }
    fn write(&self) -> Option<&Write> { None }
    fn deferred_i32(&self) -> Option<&Deferred<i32>> { None }
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

pub struct Process {
    arch: ArchProcess,
    phys: Arc<PhysicalBitmap>,
    user_virt: VirtualTree,
    kernel_virt: Arc<VirtualTree>,
    state: Mutex<ProcessState>
}

impl Process {
    pub fn new(phys: Arc<PhysicalBitmap>, kernel_virt: Arc<VirtualTree>) -> Result<Self> {
        let arch = try!(ArchProcess::new(phys.clone()));
        let user_virt = VirtualTree::new();
        user_virt.reserve(unsafe { slice::from_raw_parts_mut(0 as *mut u8, 4096) });

        Ok(Process {
            arch: arch,
            phys: phys,
            user_virt: user_virt,
            kernel_virt: kernel_virt,
            state: Mutex::new(ProcessState::new())
        })
    }

    pub fn spawn(&self) -> Result<Self> {
        Process::new(self.phys.clone(), self.kernel_virt.clone())
    }

    pub unsafe fn switch(&self) {
        self.arch.switch();
    }

    unsafe fn alloc_inner<F: Fn(usize) -> Result<usize>>(&self, ptr_opt: Option<*mut u8>, phys: F, len: usize, user: bool, writable: bool) -> Result<*mut u8> {
        let virt = if user { &self.user_virt } else { &*self.kernel_virt };

        let ptr =
            match ptr_opt {
                Some(ptr) => {
                    if !virt.reserve(slice::from_raw_parts_mut(ptr, len)) {
                        return Err(ErrNum::InvalidArgument);
                    }

                    ptr
                }

                None => try!(virt.alloc(len)).as_mut_ptr()
            };

        let mut offset = 0;
        while offset < len  {
            let ptr = ptr.offset(offset as isize);
            let addr = try!(phys(offset));
            //log!("alloc({}): map {:p} -> {:x}", len, ptr, addr);
            try!(self.arch.map(ptr, addr, user, writable));
            offset += phys_mem::PAGE_SIZE;
        }

        Ok(ptr)
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

pub fn alloc<T>(len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    let process = thread::current_process();
    unsafe {
        let make_page = |_| process.phys.alloc_page();
        let ptr = try!(process.alloc_inner(None, make_page, len * mem::size_of::<T>(), user, writable));
        Ok(slice::from_raw_parts_mut(ptr as *mut T, len))
    }
}

pub unsafe fn alloc_at<T>(base: *mut T, len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    let process = thread::current_process();
    let make_page = |_| process.phys.alloc_page();
    let ptr = try!(process.alloc_inner(Some(base as *mut u8), make_page, len * mem::size_of::<T>(), user, writable));
    assert_eq!(base as *mut u8, ptr);
    Ok(slice::from_raw_parts_mut(base, len))
}

pub unsafe fn map_phys<T>(addr: usize, len: usize, user: bool, writable: bool) -> Result<&'static mut [T]> {
    let process = thread::current_process();
    let make_page = |offset| Ok(addr + offset);
    let ptr = try!(process.alloc_inner(None, make_page, len * mem::size_of::<T>(), user, writable));
    Ok(slice::from_raw_parts_mut(ptr as *mut T, len))
}

pub fn free(p: *mut u8) -> bool {
    thread::current_process().user_virt.free(p)
}

pub fn make_handle(obj: Arc<KObj>) -> Handle {
    let process = thread::current_process();
    let mut state = lock!(process.state);
    state.make_handle(obj)
}

pub fn resolve_handle(handle: Handle) -> Option<Arc<KObj>> {
    let process = thread::current_process();
    let state = lock!(process.state);
    state.resolve_handle(handle)
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
                let p1 = Arc::new(p.spawn().unwrap());
                let p2 = Arc::new(p.spawn().unwrap());

                let d1 = thread::spawn_remote(p1, || unsafe {
                    let slice = alloc_at(0x1000 as *mut _, 0x1000, true, true).unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 123);
                    0
                });

                let d2 = thread::spawn_remote(p2, || unsafe {
                    let slice = alloc_at(0x1000 as *mut _, 0x1000, true, true).unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 456);
                    0
                });

                d1.get();
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
