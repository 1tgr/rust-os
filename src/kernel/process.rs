use alloc::arc::Arc;
use arch::process::ArchProcess;
use core::intrinsics;
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

    pub fn alloc(&self, len: usize, user: bool, writable: bool) -> Result<&mut [u8]> {
        let virt = if user { &self.user_virt } else { &*self.kernel_virt };
        let slice = try!(virt.alloc(len));
        let mut offset = 0;
        while offset < len  {
            let ptr = unsafe { slice.as_ptr().offset(offset as isize) };
            let addr = try!(self.phys.alloc_page());
            log!("alloc({}): map {:p} -> {:x}", len, ptr, addr);
            try!(self.arch.map(ptr, addr, user, writable));
            offset += phys_mem::PAGE_SIZE;
        }

        Ok(slice)
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

test!{
    fn can_alloc() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::for_kernel());
        let p = Process::new(phys, kernel_virt).unwrap();
        p.switch();

        let len = 8192;
        let slice = p.alloc(8192, false, true).unwrap();
        let sentinel = 0xaa55;
        let mut i = 0;
        while i < len {
            unsafe {
                let ptr = slice.as_mut_ptr().offset(i as isize) as *mut u16;
                intrinsics::volatile_store(ptr, sentinel);
                assert_eq!(sentinel, intrinsics::volatile_load(ptr));
            }

            i += phys_mem::PAGE_SIZE;
        }
    }
}
