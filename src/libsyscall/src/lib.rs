#![feature(asm)]
#![no_std]

#[cfg(not(feature = "kernel"))]
extern crate libc;

#[repr(usize)]
#[derive(Debug, Eq, PartialEq)]
pub enum ErrNum {
    Utf8Error = 1,
    OutOfMemory = 2,
    InvalidHandle = 3,
    NotSupported = 4,
    FileNotFound = 5,
    InvalidArgument = 6,
}

pub type Handle = usize;
pub type Result<T> = core::result::Result<T, ErrNum>;

#[macro_use]
mod macros;

mod marshal;
mod table;

#[cfg(feature = "kernel")]
mod kernel;
#[cfg(feature = "kernel")]
pub use kernel::*;

#[cfg(not(feature = "kernel"))]
mod user;
#[cfg(not(feature = "kernel"))]
pub use crate::user::*;

pub use crate::table::*;
