#![feature(alloc)]
#![feature(collections)]
#![feature(core)]
#![feature(macro_reexport)]
#![feature(no_std)]
#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate core;

#[macro_use]
#[macro_reexport(vec, format)]
extern crate collections as core_collections;

extern crate alloc;
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
pub use core_collections::vec;
pub use core::default;
pub use core::result;
pub use core_collections::string;

pub mod collections {
    pub use core_collections::BTreeMap;
    pub use core_collections::BitSet;
    pub use core_collections::Bound;
    pub use core_collections::LinkedList;
    pub use core_collections::VecDeque;
    pub use core_collections::bit_vec;
}

pub mod sync {
    pub use alloc::arc::{Arc, Weak};
    pub use core::atomic;
}
