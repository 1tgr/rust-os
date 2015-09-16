use alloc::arc::Arc;
use arch::process::ArchProcess;
use core::intrinsics;
use core::slice;
use phys_mem::{self,PhysicalBitmap};
use prelude::*;
use virt_mem::VirtualTree;

pub struct Process {
    arch: ArchProcess,
    phys: Arc<PhysicalBitmap>,
    user_virt: VirtualTree,
    kernel_virt: Arc<VirtualTree>
}

impl Process {
    pub fn new(phys: Arc<PhysicalBitmap>, kernel_virt: Arc<VirtualTree>) -> Result<Process, &'static str> {
        let arch = try!(ArchProcess::new(phys.clone()));
        let user_virt = VirtualTree::new();
        user_virt.reserve(unsafe { slice::from_raw_parts_mut(0 as *mut u8, 4096) });

        Ok(Process {
            arch: arch,
            phys: phys,
            user_virt: user_virt,
            kernel_virt: kernel_virt
        })
    }

    pub fn switch(&self) {
        self.arch.switch();
    }

    pub fn alloc(&self, len: usize, user: bool, writable: bool) -> Result<&mut [u8], &'static str> {
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
