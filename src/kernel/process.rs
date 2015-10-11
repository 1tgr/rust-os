use alloc::arc::Arc;
use arch::process::ArchProcess;
use core::mem;
use core::slice;
use io::{AsyncRead,Read,Write};
use mutex::Mutex;
use phys_mem::{self,PhysicalBitmap};
use prelude::*;
use syscall::{Handle,Result};
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

    pub fn switch(&self) {
        self.arch.switch();
    }

    unsafe fn alloc_inner<F: Fn(usize) -> Result<usize>>(&self, phys: F, len: usize, user: bool, writable: bool) -> Result<*mut u8> {
        let virt = if user { &self.user_virt } else { &*self.kernel_virt };
        let ptr = try!(virt.alloc(len)).as_mut_ptr();
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
            let ptr = try!(self.alloc_inner(|_| self.phys.alloc_page(), len * mem::size_of::<T>(), user, writable));
            Ok(slice::from_raw_parts_mut(ptr as *mut T, len))
        }
    }

    pub unsafe fn map_phys<T>(&self, addr: usize, len: usize, user: bool, writable: bool) -> Result<&mut [T]> {
        let ptr = try!(self.alloc_inner(|offset| Ok(addr + offset), len * mem::size_of::<T>(), user, writable));
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

#[cfg(feature = "test")]
pub mod test {
    use alloc::arc::Arc;
    use core::intrinsics;
    use phys_mem::PhysicalBitmap;
    use super::*;
    use virt_mem::VirtualTree;

    test!{
        fn can_alloc() {
            let phys = Arc::new(PhysicalBitmap::parse_multiboot());
            let kernel_virt = Arc::new(VirtualTree::for_kernel());
            let p = Process::new(phys, kernel_virt).unwrap();
            p.switch();

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
        }
    }
}
