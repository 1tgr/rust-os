use ops::Deref;
use ptr::Unique;
use syscall;

pub struct OSMem(Unique<u8>);

impl OSMem {
    pub fn from_raw(ptr: *mut u8) -> Self {
        assert!(!ptr.is_null());
        OSMem(unsafe { Unique::new(ptr) })
    }
}

impl Deref for OSMem {
    type Target = *mut u8;

    fn deref(&self) -> &*mut u8 {
        &*self.0
    }
}

impl Drop for OSMem {
    fn drop(&mut self) {
        let _ = syscall::free_pages(*self.0);
    }
}
