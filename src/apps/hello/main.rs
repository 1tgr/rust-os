extern crate syscall;

use std::cmp;
use std::fmt::{self,Write};
use std::slice;
use syscall::Result;
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

fn read_line(s: &mut String) -> Result<()> {
    let v = unsafe { s.as_mut_vec() };
    let capacity = v.capacity();
    let mut slice = unsafe { slice::from_raw_parts_mut(v.as_mut_ptr(), capacity) };
    let count = try!(syscall::read(unsafe { stdin }, slice));
    unsafe { v.set_len(cmp::min(count, capacity)) };
    Ok(())
}

#[no_mangle]
pub fn main() -> i32 {
    let handle = syscall::spawn("cairo_demo").unwrap();
    let code = syscall::wait_for_exit(handle);
    let _ = syscall::close(handle);
    0
}
