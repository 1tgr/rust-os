#![feature(asm)]
#![feature(never_type)]
#![no_std]

pub type Handle = usize;

#[macro_use]
mod macros;

mod error;
mod marshal;
mod table;

#[cfg(feature = "kernel")]
pub use marshal::PackedArgs;

pub use error::{ErrNum, Result};
pub use table::*;
