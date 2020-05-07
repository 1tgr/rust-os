use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::slice;
use syscall;

pub struct OSMem(NonNull<u8>, usize);

impl OSMem {
    pub unsafe fn from_raw(ptr: *mut u8, len: usize) -> Self {
        assert!(!ptr.is_null());
        OSMem(NonNull::new_unchecked(ptr), len)
    }
}

impl Deref for OSMem {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.as_ptr(), self.1) }
    }
}

impl DerefMut for OSMem {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.0.as_ptr(), self.1) }
    }
}

impl Drop for OSMem {
    fn drop(&mut self) {
        let _ = syscall::free_pages(self.0.as_ptr());
    }
}
