#![crate_name = "std"]

#![feature(alloc)]
#![feature(allow_internal_unstable)]
#![feature(collections_bound)]
#![feature(collections)]
#![feature(const_fn)]
#![feature(core_float)]
#![feature(core_intrinsics)]
#![feature(core_panic)]
#![feature(core_slice_ext)]
#![feature(core)]
#![feature(int_error_internals)]
#![feature(lang_items)]
#![feature(libc)]
#![feature(macro_reexport)]
#![feature(no_core)]
#![feature(no_std)]
#![feature(raw)]
#![feature(reflect_marker)]
#![feature(str_char)]
#![feature(unicode)]
#![feature(unique)]
#![feature(vec_push_all)]
#![feature(wrapping)]
#![feature(zero_one)]

#![no_std]

#[macro_use]
#[macro_reexport(assert, assert_eq, write, writeln)]
extern crate core as __core;

#[macro_use]
#[macro_reexport(vec, format)]
extern crate collections as core_collections;

#[macro_export]
macro_rules! try {
    ($expr:expr) => (match $expr {
        $crate::result::Result::Ok(val) => val,
        $crate::result::Result::Err(err) => {
            return $crate::result::Result::Err($crate::convert::From::from(err))
        }
    })
}

extern crate alloc;
extern crate libc;
extern crate rustc_unicode;
extern crate syscall;

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
pub use core::any;
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

pub use core::isize;
pub use core::i8;
pub use core::i16;
pub use core::i32;
pub use core::i64;

pub use core::usize;
pub use core::u8;
pub use core::u16;
pub use core::u32;
pub use core::u64;

#[path = "num/f32.rs"]   pub mod f32;
#[path = "num/f64.rs"]   pub mod f64;

pub mod error;
pub mod io;
pub mod os;

pub mod sync {
    pub use alloc::arc::{Arc, Weak};
    pub use core::sync::atomic;
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
