#![stable(feature = "rust-os", since = "1.0.0")]

use fmt;
use io::{self,Read,Write};
use os::File;
use syscall;
use syscall::libc_helpers::stdout;

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

struct StdoutWriter;

impl fmt::Write for StdoutWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match syscall::write(unsafe { stdout }, s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error)
        }
    }
}

#[stable(feature = "rust-os", since = "1.0.0")]
pub fn print(args: fmt::Arguments) {
     fmt::Write::write_fmt(&mut StdoutWriter, args).unwrap()
 }
