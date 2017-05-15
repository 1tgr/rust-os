#![stable(feature = "rust-os", since = "1.0.0")]

use ops::Deref;
use ptr::Unique;
use syscall;

#[stable(feature = "rust-os", since = "1.0.0")]
pub struct OSMem(Unique<u8>);

impl OSMem {
    #[stable(feature = "rust-os", since = "1.0.0")]
    pub fn from_raw(ptr: *mut u8) -> Self {
        assert!(!ptr.is_null());
        OSMem(unsafe { Unique::new(ptr) })
    }
}

#[stable(feature = "rust-os", since = "1.0.0")]
impl Deref for OSMem {
    type Target = Unique<u8>;

    fn deref(&self) -> &Unique<u8> {
        &self.0
    }
}

#[stable(feature = "rust-os", since = "1.0.0")]
impl Drop for OSMem {
    fn drop(&mut self) {
        let _ = syscall::free_pages(self.0.as_ptr());
    }
}
