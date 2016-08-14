#![feature(link_args)]
#![feature(start)]

extern crate syscall;

use std::fmt::{self,Write};
use syscall::{ErrNum,Result};
use syscall::libc_helpers::{stdin,stdout};

struct Writer;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match syscall::write(unsafe { stdout }, s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(std::fmt::Error)
        }
    }
}

macro_rules! print {
    ($($arg:tt)*) => { {
        let mut writer = Writer;
        let _ = write!(&mut writer, $($arg)*);
    } }
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

fn read_line() -> Result<String> {
    let mut v = Vec::new();
    loop {
        let mut buf = vec![0; 100];
        let bytes = try!(syscall::read(unsafe { stdin }, &mut buf[..]));
        if bytes < buf.len() {
            buf.truncate(bytes);
            v.extend(buf);
            break;
        }

        v.extend(buf);
    }

    String::from_utf8(v).map_err(|_| ErrNum::Utf8Error)
}

#[cfg_attr(target_arch = "x86_64", link_args = "-T ../../libsyscall/arch/amd64/link.ld ../../newlib/x86_64-elf/lib/libc.a -z max-page-size=0x1000")]
extern {
}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    let inherit = unsafe { [ stdin, stdout ] };
    loop {
        print!("> ");

        match read_line() {
            Ok(line) => {
                if line == "exit" {
                    return 0;
                } else if line.len() > 0 {
                    let handle = syscall::spawn(&line, &inherit).unwrap();
                    let _ = syscall::wait_for_exit(handle);
                    let _ = syscall::close(handle);
                }
            },

            Err(code) => { return -(code as isize) }
        }
    }
}
