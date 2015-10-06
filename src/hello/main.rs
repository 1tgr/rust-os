#![feature(lang_items)]

extern crate syscall;

use std::cmp;
use std::fmt::{self,Write};
use std::slice;
use syscall::{Handle,Result};

pub mod unwind;

static mut stdin: Handle = 0;
static mut stdout: Handle = 0;

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

fn main() -> i32 {
    let mut name = String::with_capacity(20);
    print!("Hello, what is your name? ");

    let _ = read_line(&mut name);
    println!("Hello, {}!", name);
    0x1234
}

fn init() -> Result<()> {
    unsafe {
        stdin = try!(syscall::open("stdin"));
        stdout = try!(syscall::open("stdout"));
    }

    Ok(())
}

#[no_mangle]
#[link_section=".init"]
pub extern fn start() {
    let code =
        match init() {
            Ok(()) => {
                let code = main();

                unsafe {
                    let _ = syscall::close(stdin);
                    let _ = syscall::close(stdout);
                }

                code
            },
            Err(num) => -(num as i32)
        };

    let _ = syscall::exit_thread(code);
}
