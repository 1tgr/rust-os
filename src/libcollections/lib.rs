// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Collection types.
//!
//! See [`std::collections`](../std/collections/index.html) for a detailed
//! discussion of collections in Rust.

#![crate_name = "collections"]
#![crate_type = "rlib"]
#![unstable(feature = "collections",
            reason = "library is unlikely to be stabilized with the current \
                      layout and name, use std::collections instead",
            issue = "27783")]
#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "https://doc.rust-lang.org/favicon.ico",
       html_root_url = "https://doc.rust-lang.org/nightly/",
       html_playground_url = "https://play.rust-lang.org/",
       issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/",
       test(no_crate_inject, attr(allow(unused_variables), deny(warnings))))]

#![cfg_attr(test, allow(deprecated))] // rand
#![deny(warnings)]
#![deny(missing_debug_implementations)]

#![feature(alloc)]
#![feature(allow_internal_unstable)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![cfg_attr(not(test), feature(char_escape_debug))]
#![cfg_attr(not(test), feature(core_float))]
#![feature(core_intrinsics)]
#![feature(dropck_eyepatch)]
#![feature(exact_size_is_empty)]
#![feature(fmt_internals)]
#![feature(fused)]
#![feature(generic_param_attrs)]
#![feature(heap_api)]
#![feature(i128_type)]
#![feature(inclusive_range)]
#![feature(lang_items)]
#![feature(manually_drop)]
#![feature(nonzero)]
#![feature(pattern)]
#![feature(placement_in)]
#![feature(placement_in_syntax)]
#![feature(placement_new_protocol)]
#![feature(shared)]
#![feature(slice_get_slice)]
#![feature(slice_patterns)]
#![feature(slice_rsplit)]
#![cfg_attr(not(test), feature(sort_unstable))]
#![feature(specialization)]
#![feature(staged_api)]
#![feature(str_internals)]
#![feature(str_mut_extras)]
#![feature(trusted_len)]
#![feature(unicode)]
#![feature(unique)]
#![feature(untagged_unions)]
#![cfg_attr(not(test), feature(str_checked_slicing))]
#![cfg_attr(test, feature(rand, test))]
#![feature(offset_to)]

#![no_std]

extern crate std_unicode;
extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
extern crate test;

#[doc(no_inline)]
pub use binary_heap::BinaryHeap;
#[doc(no_inline)]
pub use btree_map::BTreeMap;
#[doc(no_inline)]
pub use btree_set::BTreeSet;
#[doc(no_inline)]
pub use linked_list::LinkedList;
#[doc(no_inline)]
pub use vec_deque::VecDeque;
#[doc(no_inline)]
pub use string::String;
#[doc(no_inline)]
pub use vec::Vec;

// Needed for the vec! macro
pub use alloc::boxed;

#[macro_use]
mod macros;

pub mod binary_heap;
mod btree;
pub mod borrow;
pub mod fmt;
pub mod linked_list;
pub mod range;
pub mod slice;
pub mod str;
pub mod string;
pub mod vec;
pub mod vec_deque;

#[stable(feature = "rust1", since = "1.0.0")]
pub mod btree_map {
    //! A map based on a B-Tree.
    #[stable(feature = "rust1", since = "1.0.0")]
    pub use btree::map::*;
}

#[stable(feature = "rust1", since = "1.0.0")]
pub mod btree_set {
    //! A set based on a B-Tree.
    #[stable(feature = "rust1", since = "1.0.0")]
    pub use btree::set::*;
}

#[cfg(not(test))]
mod std {
    pub use core::ops;      // RangeFull
}

/// An endpoint of a range of keys.
///
/// # Examples
///
/// `Bound`s are range endpoints:
///
/// ```
/// #![feature(collections_range)]
///
/// use std::collections::range::RangeArgument;
/// use std::collections::Bound::*;
///
/// assert_eq!((..100).start(), Unbounded);
/// assert_eq!((1..12).start(), Included(&1));
/// assert_eq!((1..12).end(), Excluded(&12));
/// ```
///
/// Using a tuple of `Bound`s as an argument to [`BTreeMap::range`].
/// Note that in most cases, it's better to use range syntax (`1..5`) instead.
///
/// ```
/// use std::collections::BTreeMap;
/// use std::collections::Bound::{Excluded, Included, Unbounded};
///
/// let mut map = BTreeMap::new();
/// map.insert(3, "a");
/// map.insert(5, "b");
/// map.insert(8, "c");
///
/// for (key, value) in map.range((Excluded(3), Included(8))) {
///     println!("{}: {}", key, value);
/// }
///
/// assert_eq!(Some((&3, &"a")), map.range((Unbounded, Included(5))).next());
/// ```
///
/// [`BTreeMap::range`]: btree_map/struct.BTreeMap.html#method.range
#[stable(feature = "collections_bound", since = "1.17.0")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Bound<T> {
    /// An inclusive bound.
    #[stable(feature = "collections_bound", since = "1.17.0")]
    Included(T),
    /// An exclusive bound.
    #[stable(feature = "collections_bound", since = "1.17.0")]
    Excluded(T),
    /// An infinite endpoint. Indicates that there is no bound in this direction.
    #[stable(feature = "collections_bound", since = "1.17.0")]
    Unbounded,
}

/// An intermediate trait for specialization of `Extend`.
#[doc(hidden)]
trait SpecExtend<I: IntoIterator> {
    /// Extends `self` with the contents of the given iterator.
    fn spec_extend(&mut self, iter: I);
}
