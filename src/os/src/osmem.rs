use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::slice;
use syscall;

pub struct OSMem<T>(NonNull<T>, usize);

unsafe impl<T> Send for OSMem<T> where T: Send {}

impl<T> OSMem<T>
where
    T: Copy,
{
    pub unsafe fn from_raw(ptr: *mut T, len: usize) -> Self {
        assert!(!ptr.is_null());
        OSMem(NonNull::new_unchecked(ptr), len)
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
