use super::{OSHandle, Result};
use syscall;

pub struct File(OSHandle);

impl File {
    pub fn from_raw(handle: OSHandle) -> Self {
        Self(handle)
    }

    pub fn open(filename: &str) -> Result<Self> {
        Ok(Self(OSHandle::from_raw(syscall::open(filename)?)))
    }

    pub fn create_pipe() -> Self {
        Self(OSHandle::from_raw(syscall::create_pipe()))
    }

    pub fn handle(&self) -> &OSHandle {
        &self.0
    }

    pub fn duplicate(&self) -> Result<Self> {
        Ok(Self(self.0.duplicate()?))
    }
}
