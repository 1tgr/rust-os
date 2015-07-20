#![feature(asm)]
#![feature(core)]
#![feature(lang_items)]

#[macro_use] extern crate core;

use std::fmt::{Result,Write};
use std::str;

pub mod unwind;

unsafe fn syscall(num: u32, arg1: usize, arg2: usize) -> usize {
    let result;
    asm!("syscall"
         : "={rax}"(result)
         : "{rax}"(num), "{rdi}"(arg1), "{rsi}"(arg2)
         : "{rcx}", "{r11}", "cc",      // syscall/sysret clobbers rcx, r11, rflags
           "memory" 
         : "volatile");
    result
}

fn write(s: &[u8]) {
    unsafe { syscall(0, s.as_ptr() as usize, s.len()) };
}

fn exit_thread(code: u32) -> ! {
    unsafe { syscall(1, code as usize, 0) };
    unreachable!()
}

fn read_line(buf: &mut [u8]) -> usize {
    unsafe { syscall(2, buf.as_mut_ptr() as usize, buf.len()) }
}

struct Writer;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        write(s.as_bytes());
        Ok(())
    }
} 

#[no_mangle]
#[link_section=".init"]
pub extern fn start() {
    let mut writer = Writer;
    let _ = write!(&mut writer, "Hello, what is your name? ");
    let mut name = [255; 20];
    let name: &mut [u8] = &mut name;
    let count = read_line(name);
    let _ = writeln!(&mut writer, "");
    let name = &name[0 .. count];
    let _ = writeln!(&mut writer, "Hello, {}!", str::from_utf8(name).unwrap());
    exit_thread(0x1234);
}
