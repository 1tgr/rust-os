#![crate_name = "syscall"]

#![feature(asm)]
#![feature(core)]
#![feature(no_std)]
#![no_std]

#[macro_use] extern crate core;

#[cfg(not(feature = "kernel"))]
mod user {
    use core::slice::SliceExt;
    use core::str::StrExt;

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

    pub fn write(s: &str) {
        unsafe { syscall(0, s.as_ptr() as usize, s.len()) };
    }

    pub fn exit_thread(code: u32) -> ! {
        unsafe { syscall(1, code as usize, 0) };
        unreachable!()
    }

    pub fn read_line(buf: &mut [u8]) -> usize {
        unsafe { syscall(2, buf.as_mut_ptr() as usize, buf.len()) }
    }
}

#[cfg(feature = "kernel")]
pub mod kernel {
    use core::slice;
    use core::str;

    pub trait Handler {
        fn write(&self, s: &str);
        fn exit_thread(&self, code: u32) -> !;
        fn read_line(&self, buf: &mut [u8]) -> usize;
    }

    pub trait Dispatch {
        fn dispatch(&self, rax: usize, rdi: usize, rsi: usize) -> usize;
    }

    pub struct Dispatcher<T> {
        handler: T
    }

    impl<T> Dispatcher<T> {
        pub fn new(handler: T) -> Dispatcher<T> {
            Dispatcher {
                handler: handler
            }
        }
    }

    impl<T> Dispatch for Dispatcher<T> where T : Handler {
        fn dispatch(&self, rax: usize, rdi: usize, rsi: usize) -> usize {
            match rax {
                0 => { self.handler.write(str::from_utf8(unsafe { slice::from_raw_parts(rdi as *const _, rsi) }).unwrap()); 0 },
                1 => self.handler.exit_thread(rdi as u32),
                2 => self.handler.read_line(unsafe { slice::from_raw_parts_mut(rdi as *mut _, rsi) } ),
                _ => 0
            }
        }
    }
}

#[cfg(not(feature = "kernel"))]
pub use user::*;
