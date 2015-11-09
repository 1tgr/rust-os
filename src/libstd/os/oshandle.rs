use ops::Deref;
use syscall::{self,Handle};

pub struct OSHandle(Handle);

impl OSHandle {
    pub fn from_raw(handle: Handle) -> Self {
        OSHandle(handle)
    }
}

impl Deref for OSHandle {
    type Target = Handle;

    fn deref(&self) -> &Handle {
        &self.0
    }
}

impl Drop for OSHandle {
    fn drop(&mut self) {
        let _ = syscall::close(self.0);
    }
}
