#![crate_name = "syscall"]

#![feature(asm)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(no_std)]
#![no_std]

#![cfg_attr(not(feature = "kernel"), feature(libc))]

#[cfg(not(feature = "kernel"))]
extern crate libc;

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
pub mod libc_helpers;
