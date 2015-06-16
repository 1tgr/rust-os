use ::arch::process::ArchProcess;
use ::phys_mem::{self,PhysicalBitmap};
use ::virt_mem::VirtualTree;
use std::intrinsics;
use std::result::Result::{self,Ok};
use std::sync::Arc;

pub struct Process {
    arch: ArchProcess,
    phys: Arc<PhysicalBitmap>,
    user_virt: VirtualTree,
    kernel_virt: Arc<VirtualTree>
}

impl Process {
    pub fn new(phys: Arc<PhysicalBitmap>, kernel_virt: Arc<VirtualTree>) -> Process {
        let user_virt = VirtualTree::new();
        user_virt.reserve(0 as *mut u8, 4096);

        Process {
            arch: ArchProcess::new(),
            phys: phys,
            user_virt: user_virt,
            kernel_virt: kernel_virt
        }
    }

    pub fn kernel(phys: Arc<PhysicalBitmap>, kernel_virt: Arc<VirtualTree>) -> Process {
        let user_virt = VirtualTree::new();
        user_virt.reserve(0 as *mut u8, 4096);

        Process {
            arch: ArchProcess::kernel(),
            phys: phys,
            user_virt: user_virt,
            kernel_virt: kernel_virt
        }
    }

    pub fn alloc(&self, len: usize, user: bool, writable: bool) -> Result<*mut u8, &'static str> {
        let virt = if user { &self.user_virt } else { &*self.kernel_virt };
        let base_ptr = try!(virt.alloc(len));
        let alloc = || self.phys.alloc_page().unwrap();
        let mut offset = 0;
        while offset < len  {
            let ptr = unsafe { base_ptr.offset(offset as isize) };
            let addr = try!(self.phys.alloc_page());
            self.arch.map(&alloc, ptr, addr, user, writable);
            offset += phys_mem::PAGE_SIZE;
        }

        Ok(base_ptr)
    }
}

test!{
    fn can_alloc() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Process::new(phys, kernel_virt);
        let len = 8192;
        let base_ptr = p.alloc(8192, false, true).unwrap();
        let sentinel = 0xaa55;
        let mut i = 0;
        while i < len {
            unsafe {
                let ptr = base_ptr.offset(i as isize) as *mut u16;
                intrinsics::volatile_store(ptr, sentinel);
                assert_eq!(sentinel, intrinsics::volatile_load(ptr));
            }

            i += phys_mem::PAGE_SIZE;
        }
    }
}
