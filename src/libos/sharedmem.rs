use super::{OSHandle, OSMem, Result};
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
    ptr: Option<OSMem>,
}

impl SharedMem {
    pub fn from_raw(handle: OSHandle, writable: bool) -> Self {
        SharedMem {
            handle,
            writable,
            ptr: None,
        }
    }

    pub fn new(writable: bool) -> Result<Self> {
        let handle = OSHandle::from_raw(syscall::create_shared_mem()?);
        Ok(SharedMem::from_raw(handle, writable))
    }

    pub fn handle(&self) -> &OSHandle {
        &self.handle
    }

    pub fn resize(&mut self, new_len: usize) -> Result<()> {
        let old_len = self.ptr.as_ref().map_or(0, |ptr| ptr.len());
        if align_up(new_len, 4096) == align_up(old_len, 4096) {
            return Ok(());
        }

        if new_len == 0 {
            self.ptr = None;
        } else {
            let ptr = syscall::map_shared_mem(self.handle.get(), new_len, self.writable)?;
            self.ptr = Some(unsafe { OSMem::from_raw(ptr, new_len) });
        }

        Ok(())
    }
}

impl Deref for SharedMem {
    type Target = OSMem;

    fn deref(&self) -> &OSMem {
        self.ptr.as_ref().unwrap()
    }
}

impl DerefMut for SharedMem {
    fn deref_mut(&mut self) -> &mut OSMem {
        self.ptr.as_mut().unwrap()
    }
}
