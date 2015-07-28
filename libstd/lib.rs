#![crate_name = "std"]

#![feature(alloc)]
#![feature(allow_internal_unstable)]
#![feature(collections)]
#![feature(collections_bound)]
#![feature(core)]
#![feature(core_float)]
#![feature(core_intrinsics)]
#![feature(core_panic)]
#![feature(lang_items)]
#![feature(macro_reexport)]
#![feature(no_std)]
#![feature(raw)]
#![no_std]

#[macro_use]
#[macro_reexport(assert, assert_eq, try, write, writeln)]
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

#[path = "num/float_macros.rs"]
#[macro_use]
mod float_macros;

#[path = "num/int_macros.rs"]
#[macro_use]
mod int_macros;

#[path = "num/uint_macros.rs"]
#[macro_use]
mod uint_macros;

#[path = "num/isize.rs"]  pub mod isize;
#[path = "num/i8.rs"]   pub mod i8;
#[path = "num/i16.rs"]  pub mod i16;
#[path = "num/i32.rs"]  pub mod i32;
#[path = "num/i64.rs"]  pub mod i64;

#[path = "num/usize.rs"] pub mod usize;
#[path = "num/u8.rs"]   pub mod u8;
#[path = "num/u16.rs"]  pub mod u16;
#[path = "num/u32.rs"]  pub mod u32;
#[path = "num/u64.rs"]  pub mod u64;

#[path = "num/f32.rs"]   pub mod f32;
#[path = "num/f64.rs"]   pub mod f64;

pub mod num;

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
pub use core::panicking;
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
}

pub mod sync {
    pub use alloc::arc::{Arc, Weak};
    pub use core::atomic;
}

#[macro_export]
#[allow_internal_unstable]
macro_rules! panic {
    () => (
        panic!("explicit panic")
    );
    ($msg:expr) => ({
        static _MSG_FILE_LINE: (&'static str, &'static str, u32) = ($msg, file!(), line!());
        $crate::panicking::panic(&_MSG_FILE_LINE)
    });
    ($fmt:expr, $($arg:tt)*) => ({
        // The leading _'s are to avoid dead code warnings if this is
        // used inside a dead function. Just `#[allow(dead_code)]` is
        // insufficient, since the user may have
        // `#[forbid(dead_code)]` and which cannot be overridden.
        static _FILE_LINE: (&'static str, u32) = (file!(), line!());
        $crate::panicking::panic_fmt(format_args!($fmt, $($arg)*), &_FILE_LINE)
    });
}
