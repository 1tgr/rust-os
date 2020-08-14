use crate::{OSHandle, OSMem, Result};
use core::ops::{Deref, DerefMut};
use syscall;

fn align_down(value: usize, round: usize) -> usize {
    value & !(round - 1)
}

fn align_up(value: usize, round: usize) -> usize {
    align_down(value + round - 1, round)
}

pub struct SharedMem {
    handle: OSHandle,
    writable: bool,
    mem: OSMem,
}

impl SharedMem {
    pub fn from_raw(handle: OSHandle, len: usize, writable: bool) -> Result<Self> {
        let ptr = syscall::map_shared_mem(handle.get(), len, writable)?;
        let mem = unsafe { OSMem::from_raw(ptr, len) };
        Ok(Self { handle, writable, mem })
    }

    pub fn new(len: usize, writable: bool) -> Result<Self> {
        let handle = OSHandle::from_raw(syscall::create_shared_mem());
        Self::from_raw(handle, len, writable)
    }

    pub fn as_handle(&self) -> &OSHandle {
        &self.handle
    }

    pub fn into_inner(self) -> (OSHandle, OSMem) {
        (self.handle, self.mem)
    }

    pub fn resize(&mut self, new_len: usize) -> Result<()> {
        let old_len = self.mem.len();
        if align_up(new_len, 4096) == align_up(old_len, 4096) {
            return Ok(());
        }

        let ptr = syscall::map_shared_mem(self.handle.get(), new_len, self.writable)?;
        self.mem = unsafe { OSMem::from_raw(ptr, new_len) };
        Ok(())
    }
}

impl Deref for SharedMem {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.mem.as_ref()
    }
}

impl DerefMut for SharedMem {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.mem.as_mut()
    }
}
