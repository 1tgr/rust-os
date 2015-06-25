use ::arch::mmu::AddressSpace;
use ::phys_mem::PhysicalBitmap;
use std::sync::Arc;

pub struct ArchProcess {
    address_space: AddressSpace
}

impl ArchProcess {
    pub fn new(bitmap: Arc<PhysicalBitmap>) -> Result<ArchProcess, &'static str> {
        Ok(ArchProcess {
            address_space: try!(AddressSpace::new(bitmap))
        })
    }

    pub fn switch(&self) {
        self.address_space.switch()
    }

    pub fn map<T>(&self, ptr: *const T, addr: usize, user: bool, writable: bool) -> Result<(), &'static str> {
        self.address_space.map(ptr, addr, user, writable)
    }
}
