#![feature(asm)]
#![feature(core)]
#![feature(lang_items)]

#[macro_use] extern crate core;
extern crate syscall;

use std::fmt::{Result,Write};
use std::str;

pub mod unwind;

struct Writer;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        match syscall::write(s) {
            Ok(()) => Ok(()),
            Err(_) => Err(std::fmt::Error)
        }
    }
} 

#[no_mangle]
#[link_section=".init"]
pub extern fn start() {
    let mut writer = Writer;
    let _ = write!(&mut writer, "Hello, what is your name? ");
    let mut name = [255; 20];
    let name: &mut [u8] = &mut name;
    let count = syscall::read_line(name).unwrap_or(0);
    let _ = writeln!(&mut writer, "");
    let name = &name[0 .. count];
    let _ = writeln!(&mut writer, "Hello, {}!", str::from_utf8(name).unwrap());
    let _ = syscall::exit_thread(0x1234);
}
