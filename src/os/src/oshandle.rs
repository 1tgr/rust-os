use syscall::{self, Handle, Result};

pub struct OSHandle(Handle);

impl OSHandle {
    pub fn from_raw(handle: Handle) -> Self {
        Self(handle)
    }

    pub fn get(&self) -> Handle {
        self.0
    }

    pub fn duplicate(&self) -> Result<Self> {
        Ok(Self(syscall::duplicate_handle(self.0)?))
    }
}

impl Drop for OSHandle {
    fn drop(&mut self) {
        let _ = syscall::close(self.0);
    }
}
