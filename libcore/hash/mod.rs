// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generic hashing support.
//!
//! This module provides a generic way to compute the hash of a value. The
//! simplest way to make a type hashable is to use `#[derive(Hash)]`:
//!
//! # Examples
//!
//! ```rust
//! # #![feature(hash)]
//! use std::hash::{hash, Hash, SipHasher};
//!
//! #[derive(Hash)]
//! struct Person {
//!     id: u32,
//!     name: String,
//!     phone: u64,
//! }
//!
//! let person1 = Person { id: 5, name: "Janet".to_string(), phone: 555_666_7777 };
//! let person2 = Person { id: 5, name: "Bob".to_string(), phone: 555_666_7777 };
//!
//! assert!(hash::<_, SipHasher>(&person1) != hash::<_, SipHasher>(&person2));
//! ```
//!
//! If you need more control over how a value is hashed, you need to implement
//! the trait `Hash`:
//!
//! ```rust
//! # #![feature(hash)]
//! use std::hash::{hash, Hash, Hasher, SipHasher};
//!
//! struct Person {
//!     id: u32,
//!     name: String,
//!     phone: u64,
//! }
//!
//! impl Hash for Person {
//!     fn hash<H: Hasher>(&self, state: &mut H) {
//!         self.id.hash(state);
//!         self.phone.hash(state);
//!     }
//! }
//!
//! let person1 = Person { id: 5, name: "Janet".to_string(), phone: 555_666_7777 };
//! let person2 = Person { id: 5, name: "Bob".to_string(), phone: 555_666_7777 };
//!
//! assert_eq!(hash::<_, SipHasher>(&person1), hash::<_, SipHasher>(&person2));
//! ```

#![stable(feature = "rust1", since = "1.0.0")]

use prelude::*;

use mem;

pub use self::sip::SipHasher;

mod sip;

/// A hashable type.
///
/// The `H` type parameter is an abstract hash state that is used by the `Hash`
/// to compute the hash.
///
/// If you are also implementing `Eq`, there is an additional property that
/// is important:
///
/// ```text
/// k1 == k2 -> hash(k1) == hash(k2)
/// ```
///
/// In other words, if two keys are equal, their hashes should also be equal.
/// `HashMap` and `HashSet` both rely on this behavior.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Hash {
    /// Feeds this value into the state given, updating the hasher as necessary.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn hash<H: Hasher>(&self, state: &mut H);

    /// Feeds a slice of this type into the state provided.
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) where Self: Sized {
        for piece in data {
            piece.hash(state);
        }
    }
}

/// A trait which represents the ability to hash an arbitrary stream of bytes.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Hasher {
    /// Completes a round of hashing, producing the output hash generated.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn finish(&self) -> u64;

    /// Writes some data into this `Hasher`
    #[stable(feature = "rust1", since = "1.0.0")]
    fn write(&mut self, bytes: &[u8]);

    /// Write a single `u8` into this hasher
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_u8(&mut self, i: u8) { self.write(&[i]) }
    /// Write a single `u16` into this hasher.
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_u16(&mut self, i: u16) {
        self.write(&unsafe { mem::transmute::<_, [u8; 2]>(i) })
    }
    /// Write a single `u32` into this hasher.
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_u32(&mut self, i: u32) {
        self.write(&unsafe { mem::transmute::<_, [u8; 4]>(i) })
    }
    /// Write a single `u64` into this hasher.
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_u64(&mut self, i: u64) {
        self.write(&unsafe { mem::transmute::<_, [u8; 8]>(i) })
    }
    /// Write a single `usize` into this hasher.
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_usize(&mut self, i: usize) {
        if cfg!(target_pointer_width = "32") {
            self.write_u32(i as u32)
        } else {
            self.write_u64(i as u64)
        }
    }

    /// Write a single `i8` into this hasher.
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_i8(&mut self, i: i8) { self.write_u8(i as u8) }
    /// Write a single `i16` into this hasher.
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_i16(&mut self, i: i16) { self.write_u16(i as u16) }
    /// Write a single `i32` into this hasher.
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_i32(&mut self, i: i32) { self.write_u32(i as u32) }
    /// Write a single `i64` into this hasher.
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_i64(&mut self, i: i64) { self.write_u64(i as u64) }
    /// Write a single `isize` into this hasher.
    #[inline]
    #[unstable(feature = "hash", reason = "module was recently redesigned")]
    fn write_isize(&mut self, i: isize) { self.write_usize(i as usize) }
}

/// Hash a value with the default SipHasher algorithm (two initial keys of 0).
///
/// The specified value will be hashed with this hasher and then the resulting
/// hash will be returned.
#[unstable(feature = "hash", reason = "module was recently redesigned")]
pub fn hash<T: Hash, H: Hasher + Default>(value: &T) -> u64 {
    let mut h: H = Default::default();
    value.hash(&mut h);
    h.finish()
}

//////////////////////////////////////////////////////////////////////////////

mod impls {
    use prelude::*;

    use slice;
    use super::*;

    macro_rules! impl_write {
        ($(($ty:ident, $meth:ident),)*) => {$(
            #[stable(feature = "rust1", since = "1.0.0")]
            impl Hash for $ty {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    state.$meth(*self)
                }

                fn hash_slice<H: Hasher>(data: &[$ty], state: &mut H) {
                    // FIXME(#23542) Replace with type ascription.
                    #![allow(trivial_casts)]
                    let newlen = data.len() * ::$ty::BYTES;
                    let ptr = data.as_ptr() as *const u8;
                    state.write(unsafe { slice::from_raw_parts(ptr, newlen) })
                }
            }
        )*}
    }

    impl_write! {
        (u8, write_u8),
        (u16, write_u16),
        (u32, write_u32),
        (u64, write_u64),
        (usize, write_usize),
        (i8, write_i8),
        (i16, write_i16),
        (i32, write_i32),
        (i64, write_i64),
        (isize, write_isize),
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Hash for bool {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u8(*self as u8)
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Hash for char {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u32(*self as u32)
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Hash for str {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write(self.as_bytes());
            state.write_u8(0xff)
        }
    }

    macro_rules! impl_hash_tuple {
        () => (
            #[stable(feature = "rust1", since = "1.0.0")]
            impl Hash for () {
                fn hash<H: Hasher>(&self, _state: &mut H) {}
            }
        );

        ( $($name:ident)+) => (
            #[stable(feature = "rust1", since = "1.0.0")]
            impl<$($name: Hash),*> Hash for ($($name,)*) {
                #[allow(non_snake_case)]
                fn hash<S: Hasher>(&self, state: &mut S) {
                    let ($(ref $name,)*) = *self;
                    $($name.hash(state);)*
                }
            }
        );
    }

    impl_hash_tuple! {}
    impl_hash_tuple! { A }
    impl_hash_tuple! { A B }
    impl_hash_tuple! { A B C }
    impl_hash_tuple! { A B C D }
    impl_hash_tuple! { A B C D E }
    impl_hash_tuple! { A B C D E F }
    impl_hash_tuple! { A B C D E F G }
    impl_hash_tuple! { A B C D E F G H }
    impl_hash_tuple! { A B C D E F G H I }
    impl_hash_tuple! { A B C D E F G H I J }
    impl_hash_tuple! { A B C D E F G H I J K }
    impl_hash_tuple! { A B C D E F G H I J K L }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl<T: Hash> Hash for [T] {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.len().hash(state);
            Hash::hash_slice(self, state)
        }
    }


    #[stable(feature = "rust1", since = "1.0.0")]
    impl<'a, T: ?Sized + Hash> Hash for &'a T {
        fn hash<H: Hasher>(&self, state: &mut H) {
            (**self).hash(state);
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl<'a, T: ?Sized + Hash> Hash for &'a mut T {
        fn hash<H: Hasher>(&self, state: &mut H) {
            (**self).hash(state);
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl<T> Hash for *const T {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_usize(*self as usize)
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl<T> Hash for *mut T {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_usize(*self as usize)
        }
    }
}
