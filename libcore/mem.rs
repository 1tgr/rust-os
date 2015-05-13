// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Basic functions for dealing with memory
//!
//! This module contains functions for querying the size and alignment of
//! types, initializing and manipulating memory.

#![stable(feature = "rust1", since = "1.0.0")]

use marker::Sized;
use intrinsics;
use ptr;

#[stable(feature = "rust1", since = "1.0.0")]
pub use intrinsics::transmute;

/// Leaks a value into the void, consuming ownership and never running its
/// destructor.
///
/// This function will take ownership of its argument, but is distinct from the
/// `mem::drop` function in that it **does not run the destructor**, leaking the
/// value and any resources that it owns.
///
/// # Safety
///
/// This function is not marked as `unsafe` as Rust does not guarantee that the
/// `Drop` implementation for a value will always run. Note, however, that
/// leaking resources such as memory or I/O objects is likely not desired, so
/// this function is only recommended for specialized use cases.
///
/// The safety of this function implies that when writing `unsafe` code
/// yourself care must be taken when leveraging a destructor that is required to
/// run to preserve memory safety. There are known situations where the
/// destructor may not run (such as if ownership of the object with the
/// destructor is returned) which must be taken into account.
///
/// # Other forms of Leakage
///
/// It's important to point out that this function is not the only method by
/// which a value can be leaked in safe Rust code. Other known sources of
/// leakage are:
///
/// * `Rc` and `Arc` cycles
/// * `mpsc::{Sender, Receiver}` cycles (they use `Arc` internally)
/// * Panicking destructors are likely to leak local resources
///
/// # Example
///
/// ```rust,no_run
/// use std::mem;
/// use std::fs::File;
///
/// // Leak some heap memory by never deallocating it
/// let heap_memory = Box::new(3);
/// mem::forget(heap_memory);
///
/// // Leak an I/O object, never closing the file
/// let file = File::open("foo.txt").unwrap();
/// mem::forget(file);
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn forget<T>(t: T) {
    unsafe { intrinsics::forget(t) }
}

/// Returns the size of a type in bytes.
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// assert_eq!(4, mem::size_of::<i32>());
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn size_of<T>() -> usize {
    unsafe { intrinsics::size_of::<T>() }
}

/// Returns the size of the type that `_val` points to in bytes.
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// assert_eq!(4, mem::size_of_val(&5i32));
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn size_of_val<T>(_val: &T) -> usize {
    size_of::<T>()
}

/// Returns the ABI-required minimum alignment of a type
///
/// This is the alignment used for struct fields. It may be smaller than the preferred alignment.
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// assert_eq!(4, mem::min_align_of::<i32>());
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn min_align_of<T>() -> usize {
    unsafe { intrinsics::min_align_of::<T>() }
}

/// Returns the ABI-required minimum alignment of the type of the value that `_val` points to
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// assert_eq!(4, mem::min_align_of_val(&5i32));
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn min_align_of_val<T>(_val: &T) -> usize {
    min_align_of::<T>()
}

/// Returns the alignment in memory for a type.
///
/// This function will return the alignment, in bytes, of a type in memory. If the alignment
/// returned is adhered to, then the type is guaranteed to function properly.
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// assert_eq!(4, mem::align_of::<i32>());
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn align_of<T>() -> usize {
    // We use the preferred alignment as the default alignment for a type. This
    // appears to be what clang migrated towards as well:
    //
    // http://lists.cs.uiuc.edu/pipermail/cfe-commits/Week-of-Mon-20110725/044411.html
    unsafe { intrinsics::pref_align_of::<T>() }
}

/// Returns the alignment of the type of the value that `_val` points to.
///
/// This is similar to `align_of`, but function will properly handle types such as trait objects
/// (in the future), returning the alignment for an arbitrary value at runtime.
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// assert_eq!(4, mem::align_of_val(&5i32));
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn align_of_val<T>(_val: &T) -> usize {
    align_of::<T>()
}

/// Creates a value initialized to zero.
///
/// This function is similar to allocating space for a local variable and zeroing it out (an unsafe
/// operation).
///
/// Care must be taken when using this function, if the type `T` has a destructor and the value
/// falls out of scope (due to unwinding or returning) before being initialized, then the
/// destructor will run on zeroed data, likely leading to crashes.
///
/// This is useful for FFI functions sometimes, but should generally be avoided.
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// let x: i32 = unsafe { mem::zeroed() };
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn zeroed<T>() -> T {
    intrinsics::init()
}

/// Creates a value initialized to an unspecified series of bytes.
///
/// The byte sequence usually indicates that the value at the memory
/// in question has been dropped. Thus, *if* T carries a drop flag,
/// any associated destructor will not be run when the value falls out
/// of scope.
///
/// Some code at one time used the `zeroed` function above to
/// accomplish this goal.
///
/// This function is expected to be deprecated with the transition
/// to non-zeroing drop.
#[inline]
#[unstable(feature = "filling_drop")]
pub unsafe fn dropped<T>() -> T {
    #[inline(always)]
    unsafe fn dropped_impl<T>() -> T { intrinsics::init_dropped() }

    dropped_impl()
}

/// Creates an uninitialized value.
///
/// Care must be taken when using this function, if the type `T` has a destructor and the value
/// falls out of scope (due to unwinding or returning) before being initialized, then the
/// destructor will run on uninitialized data, likely leading to crashes.
///
/// This is useful for FFI functions sometimes, but should generally be avoided.
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// let x: i32 = unsafe { mem::uninitialized() };
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn uninitialized<T>() -> T {
    intrinsics::uninit()
}

/// Swap the values at two mutable locations of the same type, without deinitialising or copying
/// either one.
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// let x = &mut 5;
/// let y = &mut 42;
///
/// mem::swap(x, y);
///
/// assert_eq!(42, *x);
/// assert_eq!(5, *y);
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn swap<T>(x: &mut T, y: &mut T) {
    unsafe {
        // Give ourselves some scratch space to work with
        let mut t: T = uninitialized();

        // Perform the swap, `&mut` pointers never alias
        ptr::copy_nonoverlapping(&*x, &mut t, 1);
        ptr::copy_nonoverlapping(&*y, x, 1);
        ptr::copy_nonoverlapping(&t, y, 1);

        // y and t now point to the same thing, but we need to completely forget `t`
        // because it's no longer relevant.
        forget(t);
    }
}

/// Replaces the value at a mutable location with a new one, returning the old value, without
/// deinitialising or copying either one.
///
/// This is primarily used for transferring and swapping ownership of a value in a mutable
/// location.
///
/// # Examples
///
/// A simple example:
///
/// ```
/// use std::mem;
///
/// let mut v: Vec<i32> = Vec::new();
///
/// mem::replace(&mut v, Vec::new());
/// ```
///
/// This function allows consumption of one field of a struct by replacing it with another value.
/// The normal approach doesn't always work:
///
/// ```rust,ignore
/// struct Buffer<T> { buf: Vec<T> }
///
/// impl<T> Buffer<T> {
///     fn get_and_reset(&mut self) -> Vec<T> {
///         // error: cannot move out of dereference of `&mut`-pointer
///         let buf = self.buf;
///         self.buf = Vec::new();
///         buf
///     }
/// }
/// ```
///
/// Note that `T` does not necessarily implement `Clone`, so it can't even clone and reset
/// `self.buf`. But `replace` can be used to disassociate the original value of `self.buf` from
/// `self`, allowing it to be returned:
///
/// ```
/// use std::mem;
/// # struct Buffer<T> { buf: Vec<T> }
/// impl<T> Buffer<T> {
///     fn get_and_reset(&mut self) -> Vec<T> {
///         mem::replace(&mut self.buf, Vec::new())
///     }
/// }
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn replace<T>(dest: &mut T, mut src: T) -> T {
    swap(dest, &mut src);
    src
}

/// Disposes of a value.
///
/// This function can be used to destroy any value by allowing `drop` to take ownership of its
/// argument.
///
/// # Examples
///
/// ```
/// use std::cell::RefCell;
///
/// let x = RefCell::new(1);
///
/// let mut mutable_borrow = x.borrow_mut();
/// *mutable_borrow = 1;
///
/// drop(mutable_borrow); // relinquish the mutable borrow on this slot
///
/// let borrow = x.borrow();
/// println!("{}", *borrow);
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn drop<T>(_x: T) { }

macro_rules! repeat_u8_as_u32 {
    ($name:expr) => { (($name as u32) << 24 |
                       ($name as u32) << 16 |
                       ($name as u32) <<  8 |
                       ($name as u32)) }
}
macro_rules! repeat_u8_as_u64 {
    ($name:expr) => { ((repeat_u8_as_u32!($name) as u64) << 32 |
                       (repeat_u8_as_u32!($name) as u64)) }
}

// NOTE: Keep synchronized with values used in librustc_trans::trans::adt.
//
// In particular, the POST_DROP_U8 marker must never equal the
// DTOR_NEEDED_U8 marker.
//
// For a while pnkfelix was using 0xc1 here.
// But having the sign bit set is a pain, so 0x1d is probably better.
//
// And of course, 0x00 brings back the old world of zero'ing on drop.
#[unstable(feature = "filling_drop")]
pub const POST_DROP_U8: u8 = 0x1d;
#[unstable(feature = "filling_drop")]
pub const POST_DROP_U32: u32 = repeat_u8_as_u32!(POST_DROP_U8);
#[unstable(feature = "filling_drop")]
pub const POST_DROP_U64: u64 = repeat_u8_as_u64!(POST_DROP_U8);

#[cfg(target_pointer_width = "32")]
#[unstable(feature = "filling_drop")]
pub const POST_DROP_USIZE: usize = POST_DROP_U32 as usize;
#[cfg(target_pointer_width = "64")]
#[unstable(feature = "filling_drop")]
pub const POST_DROP_USIZE: usize = POST_DROP_U64 as usize;

/// Interprets `src` as `&U`, and then reads `src` without moving the contained
/// value.
///
/// This function will unsafely assume the pointer `src` is valid for
/// `sizeof(U)` bytes by transmuting `&T` to `&U` and then reading the `&U`. It
/// will also unsafely create a copy of the contained value instead of moving
/// out of `src`.
///
/// It is not a compile-time error if `T` and `U` have different sizes, but it
/// is highly encouraged to only invoke this function where `T` and `U` have the
/// same size. This function triggers undefined behavior if `U` is larger than
/// `T`.
///
/// # Examples
///
/// ```
/// use std::mem;
///
/// let one = unsafe { mem::transmute_copy(&1) };
///
/// assert_eq!(1, one);
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn transmute_copy<T, U>(src: &T) -> U {
    // FIXME(#23542) Replace with type ascription.
    #![allow(trivial_casts)]
    ptr::read(src as *const T as *const U)
}

/// Transforms lifetime of the second pointer to match the first.
#[inline]
#[unstable(feature = "core",
           reason = "this function may be removed in the future due to its \
                     questionable utility")]
pub unsafe fn copy_lifetime<'a, S: ?Sized, T: ?Sized + 'a>(_ptr: &'a S,
                                                        ptr: &T) -> &'a T {
    transmute(ptr)
}

/// Transforms lifetime of the second mutable pointer to match the first.
#[inline]
#[unstable(feature = "core",
           reason = "this function may be removed in the future due to its \
                     questionable utility")]
pub unsafe fn copy_mut_lifetime<'a, S: ?Sized, T: ?Sized + 'a>(_ptr: &'a S,
                                                               ptr: &mut T)
                                                              -> &'a mut T
{
    transmute(ptr)
}
