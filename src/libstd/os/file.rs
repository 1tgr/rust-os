use io::{self,Read,Write};
use os::{OSHandle,Result};
use syscall;

pub struct File(OSHandle);

impl File {
    pub fn from_raw(handle: OSHandle) -> Self {
        File(handle)
    }

    pub fn open(filename: &str) -> Result<Self> {
        Ok(File(OSHandle::from_raw(try!(syscall::open(filename)))))
    }

    pub fn create_pipe() -> Result<Self> {
        Ok(File(OSHandle::from_raw(try!(syscall::create_pipe()))))
    }

    pub fn handle(&self) -> &OSHandle {
        &self.0
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        syscall::read(*self.0, buf).map_err(From::from)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        syscall::write(*self.0, buf).map_err(From::from)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
