#![stable(feature = "rust-os", since = "1.0.0")]

use io::{self,Read,Write};
use os::{OSHandle,Result};
use syscall;

#[stable(feature = "rust-os", since = "1.0.0")]
pub struct File(OSHandle);

impl File {
    #[stable(feature = "rust-os", since = "1.0.0")]
    pub fn from_raw(handle: OSHandle) -> Self {
        File(handle)
    }

    #[stable(feature = "rust-os", since = "1.0.0")]
    pub fn open(filename: &str) -> Result<Self> {
        Ok(File(OSHandle::from_raw(syscall::open(filename)?)))
    }

    #[stable(feature = "rust-os", since = "1.0.0")]
    pub fn create_pipe() -> Result<Self> {
        Ok(File(OSHandle::from_raw(syscall::create_pipe()?)))
    }

    #[stable(feature = "rust-os", since = "1.0.0")]
    pub fn handle(&self) -> &OSHandle {
        &self.0
    }
}

#[stable(feature = "rust-os", since = "1.0.0")]
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        syscall::read(*self.0, buf).map_err(From::from)
    }
}

#[stable(feature = "rust-os", since = "1.0.0")]
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        syscall::write(*self.0, buf).map_err(From::from)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
