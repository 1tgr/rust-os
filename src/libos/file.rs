use super::{OSHandle,Result};
use syscall;

pub struct File(OSHandle);

impl File {
    pub fn from_raw(handle: OSHandle) -> Self {
        File(handle)
    }

    pub fn open(filename: &str) -> Result<Self> {
        Ok(File(OSHandle::from_raw(syscall::open(filename)?)))
    }

    pub fn create_pipe() -> Result<Self> {
        Ok(File(OSHandle::from_raw(syscall::create_pipe()?)))
    }

    pub fn handle(&self) -> &OSHandle {
        &self.0
    }
}
