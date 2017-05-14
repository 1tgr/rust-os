use super::{OSHandle,Result};
use syscall::{self,Handle};

pub struct Process(OSHandle);

impl Process {
    pub fn from_raw(handle: OSHandle) -> Self {
        Process(handle)
    }

    pub fn spawn(filename: &str, inherit: &[Handle]) -> Result<Self> {
        Ok(Process(OSHandle::from_raw(syscall::spawn(filename, inherit)?)))
    }

    pub fn handle(&self) -> &OSHandle {
        &self.0
    }

    pub fn wait_for_exit(&self) -> Result<()> {
        syscall::wait_for_exit(self.0.get())?;
        Ok(())
    }
}
