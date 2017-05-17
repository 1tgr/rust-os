#![stable(feature = "rust-os", since = "1.0.0")]

use fmt;
use io::{self,Read,Write};
use os::File;
use syscall;
use syscall::libc_helpers;

#[stable(feature = "rust-os", since = "1.0.0")]
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        syscall::read(self.handle().get(), buf).map_err(From::from)
    }
}

#[stable(feature = "rust-os", since = "1.0.0")]
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        syscall::write(self.handle().get(), buf).map_err(From::from)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[stable(feature = "rust-os", since = "1.0.0")]
pub fn print(args: fmt::Arguments) {
     fmt::Write::write_fmt(&mut libc_helpers::StdoutWriter, args).unwrap()
 }
