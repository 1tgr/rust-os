#![feature(alloc)]
#![feature(collections)]
#![feature(core)]
#![feature(no_std)]
#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate core;

extern crate alloc;
extern crate collections;
extern crate libc;

pub mod prelude {
    pub mod v1 {
    }
}

// #18491 - write!() expands to std::fmt::Arguments::new
pub use core::fmt;
// #16803 - #[derive] references std::cmp
pub use core::cmp;
// #21827 - Slicing syntax references std::ops
pub use core::ops;
// #21827 - Loops reference std
pub use core::iter;
pub use core::option;
// #16803 - Derive references marker/ops
pub use core::marker;
pub use alloc::*;
pub use core::clone;
pub use core::mem;

pub mod vec {
    pub use collections::vec::*;
}

