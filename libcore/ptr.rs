// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// FIXME: talk about offset, copy_memory, copy_nonoverlapping_memory

//! Operations on unsafe pointers, `*const T`, and `*mut T`.
//!
//! Working with unsafe pointers in Rust is uncommon,
//! typically limited to a few patterns.
//!
//! Use the `null` function to create null pointers, and the `is_null` method
//! of the `*const T` type  to check for null. The `*const T` type also defines
//! the `offset` method, for pointer math.
//!
//! # Common ways to create unsafe pointers
//!
//! ## 1. Coerce a reference (`&T`) or mutable reference (`&mut T`).
//!
//! ```
//! let my_num: i32 = 10;
//! let my_num_ptr: *const i32 = &my_num;
//! let mut my_speed: i32 = 88;
//! let my_speed_ptr: *mut i32 = &mut my_speed;
//! ```
//!
//! To get a pointer to a boxed value, dereference the box:
//!
//! ```
//! let my_num: Box<i32> = Box::new(10);
//! let my_num_ptr: *const i32 = &*my_num;
//! let mut my_speed: Box<i32> = Box::new(88);
//! let my_speed_ptr: *mut i32 = &mut *my_speed;
//! ```
//!
//! This does not take ownership of the original allocation
//! and requires no resource management later,
//! but you must not use the pointer after its lifetime.
//!
//! ## 2. Consume a box (`Box<T>`).
//!
//! The `into_raw` function consumes a box and returns
//! the raw pointer. It doesn't destroy `T` or deallocate any memory.
//!
//! ```
//! # #![feature(alloc)]
//! use std::boxed;
//!
//! unsafe {
//!     let my_speed: Box<i32> = Box::new(88);
//!     let my_speed: *mut i32 = boxed::into_raw(my_speed);
//!
//!     // By taking ownership of the original `Box<T>` though
//!     // we are obligated to put it together later to be destroyed.
//!     drop(Box::from_raw(my_speed));
//! }
//! ```
//!
//! Note that here the call to `drop` is for clarity - it indicates
//! that we are done with the given value and it should be destroyed.
//!
//! ## 3. Get it from C.
//!
//! ```
//! # #![feature(libc)]
//! extern crate libc;
//!
//! use std::mem;
//!
//! fn main() {
//!     unsafe {
//!         let my_num: *mut i32 = libc::malloc(mem::size_of::<i32>() as libc::size_t) as *mut i32;
//!         if my_num.is_null() {
//!             panic!("failed to allocate memory");
//!         }
//!         libc::free(my_num as *mut libc::c_void);
//!     }
//! }
//! ```
//!
//! Usually you wouldn't literally use `malloc` and `free` from Rust,
//! but C APIs hand out a lot of pointers generally, so are a common source
//! of unsafe pointers in Rust.

#![stable(feature = "rust1", since = "1.0.0")]
#![doc(primitive = "pointer")]

use mem;
use clone::Clone;
use intrinsics;
use ops::Deref;
use core::fmt;
use option::Option::{self, Some, None};
use marker::{PhantomData, Send, Sized, Sync};
use nonzero::NonZero;

use cmp::{PartialEq, Eq, Ord, PartialOrd};
use cmp::Ordering::{self, Less, Equal, Greater};

// FIXME #19649: intrinsic docs don't render, so these have no docs :(

#[stable(feature = "rust1", since = "1.0.0")]
pub use intrinsics::copy_nonoverlapping;

#[stable(feature = "rust1", since = "1.0.0")]
pub use intrinsics::copy;

#[stable(feature = "rust1", since = "1.0.0")]
pub use intrinsics::write_bytes;

/// Creates a null raw pointer.
///
/// # Examples
///
/// ```
/// use std::ptr;
///
/// let p: *const i32 = ptr::null();
/// assert!(p.is_null());
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn null<T>() -> *const T { 0 as *const T }

/// Creates a null mutable raw pointer.
///
/// # Examples
///
/// ```
/// use std::ptr;
///
/// let p: *mut i32 = ptr::null_mut();
/// assert!(p.is_null());
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn null_mut<T>() -> *mut T { 0 as *mut T }

/// Swaps the values at two mutable locations of the same type, without
/// deinitialising either. They may overlap, unlike `mem::swap` which is
/// otherwise equivalent.
///
/// # Safety
///
/// This is only unsafe because it accepts a raw pointer.
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn swap<T>(x: *mut T, y: *mut T) {
    // Give ourselves some scratch space to work with
    let mut tmp: T = mem::uninitialized();

    // Perform the swap
    copy_nonoverlapping(x, &mut tmp, 1);
    copy(y, x, 1); // `x` and `y` may overlap
    copy_nonoverlapping(&tmp, y, 1);

    // y and t now point to the same thing, but we need to completely forget `tmp`
    // because it's no longer relevant.
    mem::forget(tmp);
}

/// Replaces the value at `dest` with `src`, returning the old
/// value, without dropping either.
///
/// # Safety
///
/// This is only unsafe because it accepts a raw pointer.
/// Otherwise, this operation is identical to `mem::replace`.
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn replace<T>(dest: *mut T, mut src: T) -> T {
    mem::swap(mem::transmute(dest), &mut src); // cannot overlap
    src
}

/// Reads the value from `src` without moving it. This leaves the
/// memory in `src` unchanged.
///
/// # Safety
///
/// Beyond accepting a raw pointer, this is unsafe because it semantically
/// moves the value out of `src` without preventing further usage of `src`.
/// If `T` is not `Copy`, then care must be taken to ensure that the value at
/// `src` is not used before the data is overwritten again (e.g. with `write`,
/// `zero_memory`, or `copy_memory`). Note that `*src = foo` counts as a use
/// because it will attempt to drop the value previously at `*src`.
#[inline(always)]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn read<T>(src: *const T) -> T {
    let mut tmp: T = mem::uninitialized();
    copy_nonoverlapping(src, &mut tmp, 1);
    tmp
}

/// Reads the value from `src` and nulls it out without dropping it.
///
/// # Safety
///
/// This is unsafe for the same reasons that `read` is unsafe.
#[inline(always)]
#[unstable(feature = "core",
           reason = "may play a larger role in std::ptr future extensions")]
pub unsafe fn read_and_zero<T>(dest: *mut T) -> T {
    // Copy the data out from `dest`:
    let tmp = read(&*dest);

    // Now zero out `dest`:
    write_bytes(dest, 0, 1);

    tmp
}

/// Variant of read_and_zero that writes the specific drop-flag byte
/// (which may be more appropriate than zero).
#[inline(always)]
#[unstable(feature = "core",
           reason = "may play a larger role in std::ptr future extensions")]
pub unsafe fn read_and_drop<T>(dest: *mut T) -> T {
    // Copy the data out from `dest`:
    let tmp = read(&*dest);

    // Now mark `dest` as dropped:
    write_bytes(dest, mem::POST_DROP_U8, 1);

    tmp
}

/// Overwrites a memory location with the given value without reading or
/// dropping the old value.
///
/// # Safety
///
/// Beyond accepting a raw pointer, this operation is unsafe because it does
/// not drop the contents of `dst`. This could leak allocations or resources,
/// so care must be taken not to overwrite an object that should be dropped.
///
/// This is appropriate for initializing uninitialized memory, or overwriting
/// memory that has previously been `read` from.
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn write<T>(dst: *mut T, src: T) {
    intrinsics::move_val_init(&mut *dst, src)
}

#[stable(feature = "rust1", since = "1.0.0")]
#[lang = "const_ptr"]
impl<T: ?Sized> *const T {
    /// Returns true if the pointer is null.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_null(self) -> bool where T: Sized {
        self == 0 as *const T
    }

    /// Returns `None` if the pointer is null, or else returns a reference to
    /// the value wrapped in `Some`.
    ///
    /// # Safety
    ///
    /// While this method and its mutable counterpart are useful for
    /// null-safety, it is important to note that this is still an unsafe
    /// operation because the returned value could be pointing to invalid
    /// memory.
    #[unstable(feature = "core",
               reason = "Option is not clearly the right return type, and we may want \
                         to tie the return lifetime to a borrow of the raw pointer")]
    #[inline]
    pub unsafe fn as_ref<'a>(&self) -> Option<&'a T> where T: Sized {
        if self.is_null() {
            None
        } else {
            Some(&**self)
        }
    }

    /// Calculates the offset from a pointer. `count` is in units of T; e.g. a
    /// `count` of 3 represents a pointer offset of `3 * sizeof::<T>()` bytes.
    ///
    /// # Safety
    ///
    /// Both the starting and resulting pointer must be either in bounds or one
    /// byte past the end of an allocated object. If either pointer is out of
    /// bounds or arithmetic overflow occurs then
    /// any further use of the returned value will result in undefined behavior.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub unsafe fn offset(self, count: isize) -> *const T where T: Sized {
        intrinsics::offset(self, count)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
#[lang = "mut_ptr"]
impl<T: ?Sized> *mut T {
    /// Returns true if the pointer is null.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_null(self) -> bool where T: Sized {
        self == 0 as *mut T
    }

    /// Returns `None` if the pointer is null, or else returns a reference to
    /// the value wrapped in `Some`.
    ///
    /// # Safety
    ///
    /// While this method and its mutable counterpart are useful for
    /// null-safety, it is important to note that this is still an unsafe
    /// operation because the returned value could be pointing to invalid
    /// memory.
    #[unstable(feature = "core",
               reason = "Option is not clearly the right return type, and we may want \
                         to tie the return lifetime to a borrow of the raw pointer")]
    #[inline]
    pub unsafe fn as_ref<'a>(&self) -> Option<&'a T> where T: Sized {
        if self.is_null() {
            None
        } else {
            Some(&**self)
        }
    }

    /// Calculates the offset from a pointer. `count` is in units of T; e.g. a
    /// `count` of 3 represents a pointer offset of `3 * sizeof::<T>()` bytes.
    ///
    /// # Safety
    ///
    /// The offset must be in-bounds of the object, or one-byte-past-the-end.
    /// Otherwise `offset` invokes Undefined Behaviour, regardless of whether
    /// the pointer is used.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub unsafe fn offset(self, count: isize) -> *mut T where T: Sized {
        intrinsics::offset(self, count) as *mut T
    }

    /// Returns `None` if the pointer is null, or else returns a mutable
    /// reference to the value wrapped in `Some`.
    ///
    /// # Safety
    ///
    /// As with `as_ref`, this is unsafe because it cannot verify the validity
    /// of the returned pointer.
    #[unstable(feature = "core",
               reason = "return value does not necessarily convey all possible \
                         information")]
    #[inline]
    pub unsafe fn as_mut<'a>(&self) -> Option<&'a mut T> where T: Sized {
        if self.is_null() {
            None
        } else {
            Some(&mut **self)
        }
    }
}

// Equality for pointers
#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> PartialEq for *const T {
    #[inline]
    fn eq(&self, other: &*const T) -> bool { *self == *other }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Eq for *const T {}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> PartialEq for *mut T {
    #[inline]
    fn eq(&self, other: &*mut T) -> bool { *self == *other }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Eq for *mut T {}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Clone for *const T {
    #[inline]
    fn clone(&self) -> *const T {
        *self
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Clone for *mut T {
    #[inline]
    fn clone(&self) -> *mut T {
        *self
    }
}

// Equality for extern "C" fn pointers
mod externfnpointers {
    use mem;
    use cmp::PartialEq;

    #[stable(feature = "rust1", since = "1.0.0")]
    impl<_R> PartialEq for extern "C" fn() -> _R {
        #[inline]
        fn eq(&self, other: &extern "C" fn() -> _R) -> bool {
            let self_: *const () = unsafe { mem::transmute(*self) };
            let other_: *const () = unsafe { mem::transmute(*other) };
            self_ == other_
        }
    }
    macro_rules! fnptreq {
        ($($p:ident),*) => {
            #[stable(feature = "rust1", since = "1.0.0")]
            impl<_R,$($p),*> PartialEq for extern "C" fn($($p),*) -> _R {
                #[inline]
                fn eq(&self, other: &extern "C" fn($($p),*) -> _R) -> bool {
                    let self_: *const () = unsafe { mem::transmute(*self) };

                    let other_: *const () = unsafe { mem::transmute(*other) };
                    self_ == other_
                }
            }
        }
    }
    fnptreq! { A }
    fnptreq! { A,B }
    fnptreq! { A,B,C }
    fnptreq! { A,B,C,D }
    fnptreq! { A,B,C,D,E }
}

// Comparison for pointers
#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Ord for *const T {
    #[inline]
    fn cmp(&self, other: &*const T) -> Ordering {
        if self < other {
            Less
        } else if self == other {
            Equal
        } else {
            Greater
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> PartialOrd for *const T {
    #[inline]
    fn partial_cmp(&self, other: &*const T) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    #[inline]
    fn lt(&self, other: &*const T) -> bool { *self < *other }

    #[inline]
    fn le(&self, other: &*const T) -> bool { *self <= *other }

    #[inline]
    fn gt(&self, other: &*const T) -> bool { *self > *other }

    #[inline]
    fn ge(&self, other: &*const T) -> bool { *self >= *other }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Ord for *mut T {
    #[inline]
    fn cmp(&self, other: &*mut T) -> Ordering {
        if self < other {
            Less
        } else if self == other {
            Equal
        } else {
            Greater
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> PartialOrd for *mut T {
    #[inline]
    fn partial_cmp(&self, other: &*mut T) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    #[inline]
    fn lt(&self, other: &*mut T) -> bool { *self < *other }

    #[inline]
    fn le(&self, other: &*mut T) -> bool { *self <= *other }

    #[inline]
    fn gt(&self, other: &*mut T) -> bool { *self > *other }

    #[inline]
    fn ge(&self, other: &*mut T) -> bool { *self >= *other }
}

/// A wrapper around a raw `*mut T` that indicates that the possessor
/// of this wrapper owns the referent. This in turn implies that the
/// `Unique<T>` is `Send`/`Sync` if `T` is `Send`/`Sync`, unlike a raw
/// `*mut T` (which conveys no particular ownership semantics).  It
/// also implies that the referent of the pointer should not be
/// modified without a unique path to the `Unique` reference. Useful
/// for building abstractions like `Vec<T>` or `Box<T>`, which
/// internally use raw pointers to manage the memory that they own.
#[unstable(feature = "unique")]
pub struct Unique<T: ?Sized> {
    pointer: NonZero<*const T>,
    _marker: PhantomData<T>,
}

/// `Unique` pointers are `Send` if `T` is `Send` because the data they
/// reference is unaliased. Note that this aliasing invariant is
/// unenforced by the type system; the abstraction using the
/// `Unique` must enforce it.
#[unstable(feature = "unique")]
unsafe impl<T: Send + ?Sized> Send for Unique<T> { }

/// `Unique` pointers are `Sync` if `T` is `Sync` because the data they
/// reference is unaliased. Note that this aliasing invariant is
/// unenforced by the type system; the abstraction using the
/// `Unique` must enforce it.
#[unstable(feature = "unique")]
unsafe impl<T: Sync + ?Sized> Sync for Unique<T> { }

impl<T: ?Sized> Unique<T> {
    /// Creates a new `Unique`.
    #[unstable(feature = "unique")]
    pub unsafe fn new(ptr: *mut T) -> Unique<T> {
        Unique { pointer: NonZero::new(ptr), _marker: PhantomData }
    }

    /// Dereferences the content.
    #[unstable(feature = "unique")]
    pub unsafe fn get(&self) -> &T {
        &**self.pointer
    }

    /// Mutably dereferences the content.
    #[unstable(feature = "unique")]
    pub unsafe fn get_mut(&mut self) -> &mut T {
        &mut ***self
    }
}

#[unstable(feature = "unique")]
impl<T:?Sized> Deref for Unique<T> {
    type Target = *mut T;

    #[inline]
    fn deref<'a>(&'a self) -> &'a *mut T {
        unsafe { mem::transmute(&*self.pointer) }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Pointer for Unique<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&*self.pointer, f)
    }
}
