use alloc::arc::Arc;
use arch::process::ArchProcess;
use arch::thread as arch_thread;
use core::mem;
use core::slice::{self,bytes};
use elf::*;
use io::{AsyncRead,Read,Write};
use multiboot::multiboot_module_t;
use mutex::Mutex;
use phys_mem::{self,PhysicalBitmap};
use prelude::*;
use syscall::{ErrNum,Handle,Result};
use tar;
use thread::{self,Deferred};
use virt_mem::VirtualTree;

pub trait KObj {
    fn async_read(&self) -> Option<&AsyncRead> { None }
    fn read(&self) -> Option<&Read> { None }
    fn write(&self) -> Option<&Write> { None }
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

    pub fn alloc<T>(&self, len: usize, user: bool, writable: bool) -> Result<&mut [T]> {
        unsafe {
            let ptr = try!(self.alloc_inner(None, |_| self.phys.alloc_page(), len * mem::size_of::<T>(), user, writable));
            Ok(slice::from_raw_parts_mut(ptr as *mut T, len))
        }
    }

    pub unsafe fn alloc_at<T>(&self, base: *mut T, len: usize, user: bool, writable: bool) -> Result<&mut [T]> {
        let ptr = try!(self.alloc_inner(Some(base as *mut u8), |_| self.phys.alloc_page(), len * mem::size_of::<T>(), user, writable));
        assert_eq!(base as *mut u8, ptr);
        Ok(slice::from_raw_parts_mut(base, len))
    }

    pub unsafe fn map_phys<T>(&self, addr: usize, len: usize, user: bool, writable: bool) -> Result<&mut [T]> {
        let ptr = try!(self.alloc_inner(None, |offset| Ok(addr + offset), len * mem::size_of::<T>(), user, writable));
        Ok(slice::from_raw_parts_mut(ptr as *mut T, len))
    }

    pub fn free(&self, p: *mut u8) -> bool {
        self.user_virt.free(p)
    }

    pub fn make_handle(&self, obj: Arc<KObj>) -> Handle {
        lock!(self.state).make_handle(obj)
    }

    pub fn resolve_handle(&self, handle: Handle) -> Option<Arc<KObj>> {
        lock!(self.state).resolve_handle(handle)
    }

    pub fn close_handle(&self, handle: Handle) -> bool {
        lock!(self.state).close_handle(handle)
    }
}

pub fn spawn(executable: &str) -> Result<(Arc<Process>, Deferred<i32>)> {
    let parent = thread::current_process();
    let process = Arc::new(try!(Process::new(parent.phys.clone(), parent.kernel_virt.clone())));

    let init_in_new_process = {
        let process = process.clone();
        move || {
            let image_slice = unsafe {
                let info = phys_mem::multiboot_info();

                let mods: &[multiboot_module_t] = slice::from_raw_parts(
                    phys_mem::phys2virt(info.mods_addr as usize),
                    info.mods_count as usize);

                assert_eq!(1, mods.len());

                let mod_data: &[u8] = slice::from_raw_parts(
                    phys_mem::phys2virt(mods[0].mod_start as usize),
                    (mods[0].mod_end - mods[0].mod_start) as usize);

                tar::locate(mod_data, executable).expect("file not found") // TODO
            };

            let (entry, stack_slice) = {
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
                    let slice = unsafe { process.alloc_at::<u8>(phdr.p_vaddr as *mut u8, phdr.p_memsz as usize, true, true).unwrap() };
                    let file_slice = &image_slice[phdr.p_offset as usize .. (phdr.p_offset + phdr.p_filesz) as usize];
                    bytes::copy_memory(file_slice, slice);
                    slices.push(slice);
                }

                assert!(slices.iter().any(|ref slice| {
                    let slice_end = unsafe { slice.as_ptr().offset(slice.len() as isize) };
                    entry >= slice.as_ptr() && entry < slice_end
                }));

                let stack_slice = process.alloc(phys_mem::PAGE_SIZE * 10, true, true).unwrap();
                log!("stack_slice = 0x{:x} bytes @ {:p}", stack_slice.len(), stack_slice.as_ptr());
                (entry, stack_slice)
            };

            unsafe { arch_thread::jmp_user_mode(entry, stack_slice.as_mut_ptr().offset(stack_slice.len() as isize)) }
            // TODO: free stack
        }
    };

    Ok((process.clone(), thread::spawn_remote(process, init_in_new_process)))
}

#[cfg(feature = "test")]
pub mod test {
    use alloc::arc::Arc;
    use core::intrinsics;
    use phys_mem::PhysicalBitmap;
    use super::*;
    use thread;
    use virt_mem::VirtualTree;

    test!{
        fn can_alloc() {
            let phys = Arc::new(PhysicalBitmap::parse_multiboot());
            let kernel_virt = Arc::new(VirtualTree::for_kernel());
            let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
            thread::with_scheduler(p.clone(), || {
                let len = 4096;
                let slice = p.alloc::<u16>(len, false, true).unwrap();
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
            let phys = Arc::new(PhysicalBitmap::parse_multiboot());
            let kernel_virt = Arc::new(VirtualTree::for_kernel());
            let idle_process = Arc::new(Process::new(phys.clone(), kernel_virt.clone()).unwrap());
            thread::with_scheduler(idle_process.clone(), || {
                let p1 = Arc::new(Process::new(phys.clone(), kernel_virt.clone()).unwrap());
                let p2 = Arc::new(Process::new(phys.clone(), kernel_virt.clone()).unwrap());

                let d1 = thread::spawn_remote(p1.clone(), || unsafe {
                    let slice = p1.alloc_at(0x1000 as *mut _, 0x1000, true, true).unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 123);
                    0
                });

                let d2 = thread::spawn_remote(p2.clone(), || unsafe {
                    let slice = p2.alloc_at(0x1000 as *mut _, 0x1000, true, true).unwrap();
                    assert_eq!(0, intrinsics::volatile_load(slice.as_ptr()));
                    intrinsics::volatile_store(slice.as_mut_ptr(), 456);
                    0
                });

                d1.get();
                d2.get();
            });
        }

        /*fn kernel_addresses_are_shared() {
            let phys = Arc::new(PhysicalBitmap::parse_multiboot());
            let kernel_virt = Arc::new(VirtualTree::for_kernel());
            let idle_process = Arc::new(Process::new(phys.clone(), kernel_virt.clone()).unwrap());
            thread::with_scheduler(idle_process.clone(), || {
                let p1 = Arc::new(Process::new(phys.clone(), kernel_virt.clone()).unwrap());
                let p2 = Arc::new(Process::new(phys.clone(), kernel_virt.clone()).unwrap());
                let slice = idle_process.alloc(0x1000, false, true).unwrap();
                unsafe { intrinsics::volatile_store(slice.as_mut_ptr(), 123) };

                let d1 = thread::spawn_remote(p1.clone(), || unsafe {
                    assert_eq!(123, intrinsics::volatile_load(slice.as_ptr()));
                });

                let d2 = thread::spawn_remote(p2.clone(), || unsafe {
                    assert_eq!(123, intrinsics::volatile_load(slice.as_ptr()));
                });

                d1.get();
                d2.get();
            });
        } */
    }
}
