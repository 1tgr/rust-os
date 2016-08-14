#![feature(link_args)]
#![feature(start)]

extern crate syscall;

use std::fmt::{self,Write};
use syscall::libc_helpers::stdout;

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

#[cfg_attr(target_arch = "x86_64", link_args = "-T ../../libsyscall/arch/amd64/link.ld ../../newlib/x86_64-elf/lib/libc.a -z max-page-size=0x1000")]
extern {
}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    println!("hello world");
    0
}
