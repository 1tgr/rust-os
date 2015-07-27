#![feature(core)]
#![feature(lang_items)]

#[macro_use] extern crate core;
extern crate syscall;

use std::cmp;
use std::fmt::{self,Write};
use std::slice;
use syscall::ErrNum;

pub mod unwind;

struct Writer;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match syscall::write(s) {
            Ok(()) => Ok(()),
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

fn read_line(s: &mut String) -> Result<(), ErrNum> {
    let v = unsafe { s.as_mut_vec() };
    let capacity = v.capacity();
    let mut slice = unsafe { slice::from_raw_parts_mut(v.as_mut_ptr(), capacity) };
    let count = try!(syscall::read_line(slice));
    unsafe { v.set_len(cmp::min(count, capacity)) };
    Ok(())

}

#[no_mangle]
#[link_section=".init"]
pub extern fn start() {
    let mut name = String::with_capacity(20);
    print!("Hello, what is your name? ");

    let _ = read_line(&mut name);
    println!("");
    println!("Hello, {}!", name);
    let _ = syscall::exit_thread(0x1234);
}
