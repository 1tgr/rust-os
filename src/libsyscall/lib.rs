#![crate_name = "syscall"]

#![feature(asm)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(no_std)]
#![no_std]

#[macro_use] mod macros;

mod marshal;
mod table;

#[cfg(not(feature = "kernel"))]
mod user;

#[cfg(feature = "kernel")]
pub mod kernel;

pub use marshal::{ErrNum,Handle,FileHandle,Result};
pub use table::*;

#[cfg(not(feature = "kernel"))]
pub mod libc {
    static mut ERRNO: u32 = 0;

    #[no_mangle]
    pub extern fn sbrk(len: usize) -> *mut u8 {
        match super::alloc_pages(len) {
            Ok(p) => p,
            Err(num) => {
                unsafe { ERRNO = num as u32; }
                0 as *mut u8
            }
        }
    }

    #[no_mangle]
    pub extern fn __assert(_file: *const u8, _line: i32, _msg: *const u8) -> ! {
        let _ = super::exit_thread(-1);
        unreachable!()
    }

    #[no_mangle]
    pub unsafe extern fn __error() -> *mut u32 {
        &mut ERRNO as *mut u32
    }
}
