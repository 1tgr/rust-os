use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::{mem, slice};
use syscall::{self, Result};

pub struct OSMem<T>(NonNull<T>, usize);

unsafe impl<T> Send for OSMem<T> where T: Send {}

impl<T> OSMem<T>
where
    T: Copy,
{
    pub fn new(len: usize) -> Result<Self> {
        let byte_len = len * mem::size_of::<T>();
        unsafe {
            let ptr = syscall::alloc_pages(byte_len)?;
            Ok(Self::from_raw(ptr as *mut T, len))
        }
    }

    pub unsafe fn from_raw(ptr: *mut T, len: usize) -> Self {
        assert!(!ptr.is_null());
        Self(NonNull::new_unchecked(ptr), len)
    }
}

impl<T> AsRef<[T]> for OSMem<T> {
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}

impl<T> AsMut<[T]> for OSMem<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<T> Deref for OSMem<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.0.as_ptr(), self.1) }
    }
}

impl<T> DerefMut for OSMem<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.0.as_ptr(), self.1) }
    }
}

impl<T> Drop for OSMem<T> {
    fn drop(&mut self) {
        let _ = syscall::free_pages(self.0.as_ptr() as *mut u8);
    }
}
