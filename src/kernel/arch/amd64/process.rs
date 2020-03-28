use crate::arch::mmu::AddressSpace;
use crate::phys_mem::PhysicalBitmap;
use alloc::sync::Arc;
use syscall::Result;

pub struct ArchProcess {
    address_space: AddressSpace,
}

impl ArchProcess {
    pub fn new(bitmap: Arc<PhysicalBitmap>) -> Result<ArchProcess> {
        Ok(ArchProcess {
            address_space: AddressSpace::new(bitmap)?,
        })
    }

    pub unsafe fn switch(&self) {
        self.address_space.switch()
    }

    pub unsafe fn map<T>(&self, ptr: *const T, addr: usize, user: bool, writable: bool) -> Result<()> {
        self.address_space.map(ptr, addr, user, writable)
    }
}
