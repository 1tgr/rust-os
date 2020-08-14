#![allow(missing_docs)]
#![stable(feature = "rust-os", since = "1.0.0")]

use crate::fmt;
use crate::io::{self, Read, Write};
use os::File;
use os::libc_helpers::{StdoutWriter, StderrWriter};
use syscall;

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
     fmt::Write::write_fmt(&mut StdoutWriter, args).unwrap()
 }

#[stable(feature = "rust-os", since = "1.0.0")]
pub fn eprint(args: fmt::Arguments) {
    fmt::Write::write_fmt(&mut StderrWriter, args).unwrap()
}
