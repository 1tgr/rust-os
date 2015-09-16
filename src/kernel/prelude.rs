/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * prelude.rs
 * - Definitions meant to be used in every module
 *
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */
 pub use alloc::boxed::Box;
 pub use core::clone::Clone;
 pub use core::cmp::{PartialEq, PartialOrd, Eq, Ord};
 pub use core::convert::{AsRef, AsMut, Into, From};
 pub use core::default::Default;
 pub use core::iter::{DoubleEndedIterator, ExactSizeIterator};
 pub use core::iter::{Iterator, Extend, IntoIterator};
 pub use core::marker::{Copy, Send, Sized, Sync};
 pub use core::mem::drop;
 pub use core::ops::{Drop, Fn, FnMut, FnOnce};
 pub use core::option::Option::{self, Some, None};
 pub use core::result::Result::{self, Ok, Err};
 pub use collections::string::{String, ToString};
 pub use collections::vec::Vec;
