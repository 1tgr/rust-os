#![crate_name = "std"]

#![feature(alloc)]
#![feature(bitvec)]
#![feature(collections)]
#![feature(collections_bound)]
#![feature(core)]
#![feature(core_intrinsics)]
#![feature(macro_reexport)]
#![feature(no_std)]
#![feature(raw)]
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
        //pub use borrow::ToOwned;
        pub use boxed::Box;
        pub use clone::Clone;
        pub use cmp::{PartialEq, PartialOrd, Eq, Ord};
        pub use convert::{AsRef, AsMut, Into, From};
        pub use default::Default;
        pub use iter::{DoubleEndedIterator, ExactSizeIterator};
        pub use iter::{Iterator, Extend, IntoIterator};
        pub use marker::{Copy, Send, Sized, Sync};
        pub use mem::drop;
        pub use ops::{Drop, Fn, FnMut, FnOnce};
        pub use option::Option::{self, Some, None};
        pub use result::Result::{self, Ok, Err};
        //pub use slice::SliceConcatExt;
        pub use string::{String, ToString};
        pub use vec::Vec;
    }
}

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
pub use core::char;
pub use core::clone;
pub use core::convert;
pub use core::default;
pub use core::hash;
pub use core::intrinsics;
pub use core::mem;
pub use core::ptr;
pub use core::raw;
pub use core::result;
pub use core::slice;
pub use core::str;
pub use core_collections::fmt;
pub use core_collections::string;
pub use core_collections::vec;

pub mod collections {
    pub use core_collections::BTreeMap;
    pub use core_collections::Bound;
    pub use core_collections::LinkedList;
    pub use core_collections::VecDeque;
    pub use core_collections::bit_vec;
}

pub mod sync {
    pub use alloc::arc::{Arc, Weak};
    pub use core::atomic;
}
