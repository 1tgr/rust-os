#![crate_name = "std"]
#![stable(feature = "dummy", since = "1.0.0")]

#![feature(alloc)]
#![feature(allow_internal_unstable)]
#![feature(collections_bound)]
#![feature(collections)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(core_panic)]
#![feature(float_extras)]
#![feature(int_error_internals)]
#![feature(lang_items)]
#![feature(macro_reexport)]
#![feature(no_core)]
#![feature(question_mark)]
#![feature(raw)]
#![feature(reflect_marker)]
#![feature(staged_api)]
#![feature(stmt_expr_attributes)]
#![feature(try_from)]
#![feature(unicode)]
#![feature(unique)]
#![feature(zero_one)]

#![no_std]

#[macro_use]
#[macro_reexport(assert, assert_eq, write, writeln)]
extern crate core as __core;

#[macro_use]
#[macro_reexport(vec, format)]
extern crate collections as core_collections;

#[macro_export]
#[stable(feature = "rust-os", since = "1.0.0")]
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

#[stable(feature = "rust-os", since = "1.0.0")]
pub mod prelude {
    #[stable(feature = "rust-os", since = "1.0.0")]
    pub mod v1 {
        //pub use borrow::ToOwned;
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use boxed::Box;
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use clone::Clone;
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use cmp::{PartialEq, PartialOrd, Eq, Ord};
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use convert::{AsRef, AsMut, Into, From};
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use default::Default;
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use iter::{DoubleEndedIterator, ExactSizeIterator};
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use iter::{Iterator, Extend, IntoIterator};
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use marker::{Copy, Send, Sized, Sync};
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use mem::drop;
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use ops::{Drop, Fn, FnMut, FnOnce};
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use option::Option::{self, Some, None};
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use result::Result::{self, Ok, Err};
        //pub use slice::SliceConcatExt;
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use string::{String, ToString};
        #[stable(feature = "rust-os", since = "1.0.0")]
        pub use vec::Vec;
    }
}

pub mod num;

// #16803 - #[derive] references std::cmp
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::cmp;
// #21827 - Slicing syntax references std::ops
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::ops;
// #21827 - Loops reference std
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::iter;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::option;
// #16803 - Derive references marker/ops
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::marker;

#[stable(feature = "rust-os", since = "1.0.0")]
pub use alloc::*;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::any;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::char;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::clone;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::convert;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::default;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::hash;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::intrinsics;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::mem;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::panicking;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::ptr;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::raw;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::result;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::slice;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::str;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core_collections::fmt;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core_collections::string;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core_collections::vec;

#[stable(feature = "rust-os", since = "1.0.0")]
pub mod collections {
    #[stable(feature = "rust-os", since = "1.0.0")]
    pub use core_collections::BTreeMap;
    #[stable(feature = "rust-os", since = "1.0.0")]
    pub use core_collections::Bound;
    #[stable(feature = "rust-os", since = "1.0.0")]
    pub use core_collections::LinkedList;
    #[stable(feature = "rust-os", since = "1.0.0")]
    pub use core_collections::VecDeque;
}

#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::isize;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::i8;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::i16;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::i32;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::i64;

#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::usize;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::u8;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::u16;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::u32;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use core::u64;

#[path = "num/f32.rs"]   pub mod f32;
#[path = "num/f64.rs"]   pub mod f64;

pub mod error;
pub mod io;
pub mod os;

mod memchr;

#[stable(feature = "dummy", since = "1.0.0")]
pub mod sync {
    #[stable(feature = "dummy", since = "1.0.0")]
    pub use alloc::arc::{Arc, Weak};
    #[stable(feature = "dummy", since = "1.0.0")]
    pub use core::sync::atomic;
}

#[macro_export]
#[allow_internal_unstable]
#[stable(feature = "rust-os", since = "1.0.0")]
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
