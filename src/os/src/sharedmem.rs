use crate::{OSHandle, OSMem, Result};
use core::mem;
use core::ops::{Deref, DerefMut};
use syscall;

fn align_down(value: usize, round: usize) -> usize {
    value & !(round - 1)
}

fn align_up(value: usize, round: usize) -> usize {
    align_down(value + round - 1, round)
}

pub struct SharedMem<T> {
    handle: OSHandle,
    writable: bool,
    mem: OSMem<T>,
}

impl<T> SharedMem<T>
where
    T: Copy,
{
    pub fn from_raw(handle: OSHandle, len: usize, writable: bool) -> Result<Self> {
        let byte_len = len * mem::size_of::<T>();
        let ptr = syscall::map_shared_mem(handle.get(), byte_len, writable)? as *mut T;
        let mem = unsafe { OSMem::from_raw(ptr, len) };
        Ok(Self { handle, writable, mem })
    }

    pub fn new(len: usize, writable: bool) -> Result<Self> {
        let handle = OSHandle::from_raw(syscall::create_shared_mem());
        Self::from_raw(handle, len, writable)
    }

    pub fn resize(&mut self, new_len: usize) -> Result<()> {
        let old_byte_len = self.mem.len() * mem::size_of::<T>();
        let new_byte_len = new_len * mem::size_of::<T>();
        if align_up(new_byte_len, 4096) == align_up(old_byte_len, 4096) {
            return Ok(());
        }

        let ptr = syscall::map_shared_mem(self.handle.get(), new_byte_len, self.writable)? as *mut T;
        self.mem = unsafe { OSMem::from_raw(ptr, new_len) };
        Ok(())
    }
}

impl<T> SharedMem<T> {
    pub fn as_handle(&self) -> &OSHandle {
        &self.handle
    }

    pub fn into_inner(self) -> (OSHandle, OSMem<T>) {
        (self.handle, self.mem)
    }
}

impl<T> Deref for SharedMem<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.mem.as_ref()
    }
}

impl<T> DerefMut for SharedMem<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.mem.as_mut()
    }
}
