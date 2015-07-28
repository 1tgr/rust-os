// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Slice management and manipulation
//!
//! For more details `std::slice`.

#![stable(feature = "rust1", since = "1.0.0")]

// How this module is organized.
//
// The library infrastructure for slices is fairly messy. There's
// a lot of stuff defined here. Let's keep it clean.
//
// Since slices don't support inherent methods; all operations
// on them are defined on traits, which are then reexported from
// the prelude for convenience. So there are a lot of traits here.
//
// The layout of this file is thus:
//
// * Slice-specific 'extension' traits and their implementations. This
//   is where most of the slice API resides.
// * Implementations of a few common traits with important slice ops.
// * Definitions of a bunch of iterators.
// * Free functions.
// * The `raw` and `bytes` submodules.
// * Boilerplate trait implementations.

use mem::transmute;
use clone::Clone;
use cmp::{Ordering, PartialEq, PartialOrd, Eq, Ord};
use cmp::Ordering::{Less, Equal, Greater};
use cmp;
use default::Default;
use intrinsics::assume;
use iter::*;
use ops::{FnMut, self, Index};
use ops::RangeFull;
use option::Option;
use option::Option::{None, Some};
use result::Result;
use result::Result::{Ok, Err};
use ptr;
use mem;
use mem::size_of;
use marker::{Send, Sync, self};
use raw::Repr;
// Avoid conflicts with *both* the Slice trait (buggy) and the `slice::raw` module.
use raw::Slice as RawSlice;


//
// Extension traits
//

/// Extension methods for slices.
#[allow(missing_docs)] // docs in libcollections
#[doc(hidden)]
#[unstable(feature = "core_slice_ext",
           reason = "stable interface provided by `impl [T]` in later crates")]
pub trait SliceExt {
    type Item;

    fn split_at<'a>(&'a self, mid: usize) -> (&'a [Self::Item], &'a [Self::Item]);
    fn iter<'a>(&'a self) -> Iter<'a, Self::Item>;
    fn split<'a, P>(&'a self, pred: P) -> Split<'a, Self::Item, P>
                    where P: FnMut(&Self::Item) -> bool;
    fn splitn<'a, P>(&'a self, n: usize, pred: P) -> SplitN<'a, Self::Item, P>
                     where P: FnMut(&Self::Item) -> bool;
    fn rsplitn<'a, P>(&'a self,  n: usize, pred: P) -> RSplitN<'a, Self::Item, P>
                      where P: FnMut(&Self::Item) -> bool;
    fn windows<'a>(&'a self, size: usize) -> Windows<'a, Self::Item>;
    fn chunks<'a>(&'a self, size: usize) -> Chunks<'a, Self::Item>;
    fn get<'a>(&'a self, index: usize) -> Option<&'a Self::Item>;
    fn first<'a>(&'a self) -> Option<&'a Self::Item>;
    fn tail<'a>(&'a self) -> &'a [Self::Item];
    fn init<'a>(&'a self) -> &'a [Self::Item];
    fn split_first<'a>(&'a self) -> Option<(&'a Self::Item, &'a [Self::Item])>;
    fn split_last<'a>(&'a self) -> Option<(&'a Self::Item, &'a [Self::Item])>;
    fn last<'a>(&'a self) -> Option<&'a Self::Item>;
    unsafe fn get_unchecked<'a>(&'a self, index: usize) -> &'a Self::Item;
    fn as_ptr(&self) -> *const Self::Item;
    fn binary_search_by<F>(&self, f: F) -> Result<usize, usize> where
        F: FnMut(&Self::Item) -> Ordering;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn get_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut Self::Item>;
    fn iter_mut<'a>(&'a mut self) -> IterMut<'a, Self::Item>;
    fn first_mut<'a>(&'a mut self) -> Option<&'a mut Self::Item>;
    fn tail_mut<'a>(&'a mut self) -> &'a mut [Self::Item];
    fn init_mut<'a>(&'a mut self) -> &'a mut [Self::Item];
    fn split_first_mut<'a>(&'a mut self) -> Option<(&'a mut Self::Item, &'a mut [Self::Item])>;
    fn split_last_mut<'a>(&'a mut self) -> Option<(&'a mut Self::Item, &'a mut [Self::Item])>;
    fn last_mut<'a>(&'a mut self) -> Option<&'a mut Self::Item>;
    fn split_mut<'a, P>(&'a mut self, pred: P) -> SplitMut<'a, Self::Item, P>
                        where P: FnMut(&Self::Item) -> bool;
    fn splitn_mut<P>(&mut self, n: usize, pred: P) -> SplitNMut<Self::Item, P>
                     where P: FnMut(&Self::Item) -> bool;
    fn rsplitn_mut<P>(&mut self,  n: usize, pred: P) -> RSplitNMut<Self::Item, P>
                      where P: FnMut(&Self::Item) -> bool;
    fn chunks_mut<'a>(&'a mut self, chunk_size: usize) -> ChunksMut<'a, Self::Item>;
    fn swap(&mut self, a: usize, b: usize);
    fn split_at_mut<'a>(&'a mut self, mid: usize) -> (&'a mut [Self::Item], &'a mut [Self::Item]);
    fn reverse(&mut self);
    unsafe fn get_unchecked_mut<'a>(&'a mut self, index: usize) -> &'a mut Self::Item;
    fn as_mut_ptr(&mut self) -> *mut Self::Item;

    fn position_elem(&self, t: &Self::Item) -> Option<usize> where Self::Item: PartialEq;

    fn rposition_elem(&self, t: &Self::Item) -> Option<usize> where Self::Item: PartialEq;

    fn contains(&self, x: &Self::Item) -> bool where Self::Item: PartialEq;

    fn starts_with(&self, needle: &[Self::Item]) -> bool where Self::Item: PartialEq;

    fn ends_with(&self, needle: &[Self::Item]) -> bool where Self::Item: PartialEq;

    fn binary_search(&self, x: &Self::Item) -> Result<usize, usize> where Self::Item: Ord;
    fn next_permutation(&mut self) -> bool where Self::Item: Ord;
    fn prev_permutation(&mut self) -> bool where Self::Item: Ord;

    fn clone_from_slice(&mut self, &[Self::Item]) -> usize where Self::Item: Clone;
}

// Use macros to be generic over const/mut
macro_rules! slice_offset {
    ($ptr:expr, $by:expr) => {{
        let ptr = $ptr;
        if size_from_ptr(ptr) == 0 {
            ::intrinsics::arith_offset(ptr as *mut i8, $by) as *mut _
        } else {
            ptr.offset($by)
        }
    }};
}

macro_rules! slice_ref {
    ($ptr:expr) => {{
        let ptr = $ptr;
        if size_from_ptr(ptr) == 0 {
            // Use a non-null pointer value
            &mut *(1 as *mut _)
        } else {
            transmute(ptr)
        }
    }};
}

impl<T> SliceExt for [T] {
    type Item = T;

    #[inline]
    fn split_at(&self, mid: usize) -> (&[T], &[T]) {
        (&self[..mid], &self[mid..])
    }

    #[inline]
    fn iter<'a>(&'a self) -> Iter<'a, T> {
        unsafe {
            let p = if mem::size_of::<T>() == 0 {
                1 as *const _
            } else {
                let p = self.as_ptr();
                assume(!p.is_null());
                p
            };

            Iter {
                ptr: p,
                end: slice_offset!(p, self.len() as isize),
                _marker: marker::PhantomData
            }
        }
    }

    #[inline]
    fn split<'a, P>(&'a self, pred: P) -> Split<'a, T, P> where P: FnMut(&T) -> bool {
        Split {
            v: self,
            pred: pred,
            finished: false
        }
    }

    #[inline]
    fn splitn<'a, P>(&'a self, n: usize, pred: P) -> SplitN<'a, T, P> where
        P: FnMut(&T) -> bool,
    {
        SplitN {
            inner: GenericSplitN {
                iter: self.split(pred),
                count: n,
                invert: false
            }
        }
    }

    #[inline]
    fn rsplitn<'a, P>(&'a self, n: usize, pred: P) -> RSplitN<'a, T, P> where
        P: FnMut(&T) -> bool,
    {
        RSplitN {
            inner: GenericSplitN {
                iter: self.split(pred),
                count: n,
                invert: true
            }
        }
    }

    #[inline]
    fn windows(&self, size: usize) -> Windows<T> {
        assert!(size != 0);
        Windows { v: self, size: size }
    }

    #[inline]
    fn chunks(&self, size: usize) -> Chunks<T> {
        assert!(size != 0);
        Chunks { v: self, size: size }
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() { Some(&self[index]) } else { None }
    }

    #[inline]
    fn first(&self) -> Option<&T> {
        if self.is_empty() { None } else { Some(&self[0]) }
    }

    #[inline]
    fn tail(&self) -> &[T] { &self[1..] }

    #[inline]
    fn split_first(&self) -> Option<(&T, &[T])> {
        if self.is_empty() { None } else { Some((&self[0], &self[1..])) }
    }

    #[inline]
    fn init(&self) -> &[T] { &self[..self.len() - 1] }

    #[inline]
    fn split_last(&self) -> Option<(&T, &[T])> {
        let len = self.len();
        if len == 0 { None } else { Some((&self[len - 1], &self[..(len - 1)])) }
    }

    #[inline]
    fn last(&self) -> Option<&T> {
        if self.is_empty() { None } else { Some(&self[self.len() - 1]) }
    }

    #[inline]
    unsafe fn get_unchecked(&self, index: usize) -> &T {
        transmute(self.repr().data.offset(index as isize))
    }

    #[inline]
    fn as_ptr(&self) -> *const T {
        self.repr().data
    }

    fn binary_search_by<F>(&self, mut f: F) -> Result<usize, usize> where
        F: FnMut(&T) -> Ordering
    {
        let mut base : usize = 0;
        let mut lim : usize = self.len();

        while lim != 0 {
            let ix = base + (lim >> 1);
            match f(&self[ix]) {
                Equal => return Ok(ix),
                Less => {
                    base = ix + 1;
                    lim -= 1;
                }
                Greater => ()
            }
            lim >>= 1;
        }
        Err(base)
    }

    #[inline]
    fn len(&self) -> usize { self.repr().len }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() { Some(&mut self[index]) } else { None }
    }

    #[inline]
    fn split_at_mut(&mut self, mid: usize) -> (&mut [T], &mut [T]) {
        unsafe {
            let self2: &mut [T] = mem::transmute_copy(&self);

            (ops::IndexMut::index_mut(self, ops::RangeTo { end: mid } ),
             ops::IndexMut::index_mut(self2, ops::RangeFrom { start: mid } ))
        }
    }

    #[inline]
    fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        unsafe {
            let p = if mem::size_of::<T>() == 0 {
                1 as *mut _
            } else {
                let p = self.as_mut_ptr();
                assume(!p.is_null());
                p
            };

            IterMut {
                ptr: p,
                end: slice_offset!(p, self.len() as isize),
                _marker: marker::PhantomData
            }
        }
    }

    #[inline]
    fn last_mut(&mut self) -> Option<&mut T> {
        let len = self.len();
        if len == 0 { return None; }
        Some(&mut self[len - 1])
    }

    #[inline]
    fn first_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() { None } else { Some(&mut self[0]) }
    }

    #[inline]
    fn tail_mut(&mut self) -> &mut [T] { &mut self[1 ..] }

    #[inline]
    fn split_first_mut(&mut self) -> Option<(&mut T, &mut [T])> {
        if self.is_empty() { None } else {
            let split = self.split_at_mut(1);
            Some((&mut split.0[0], split.1))
        }
    }

    #[inline]
    fn init_mut(&mut self) -> &mut [T] {
        let len = self.len();
        &mut self[.. (len - 1)]
    }

    #[inline]
    fn split_last_mut(&mut self) -> Option<(&mut T, &mut [T])> {
        let len = self.len();
        if len == 0 { None } else {
            let split = self.split_at_mut(len - 1);
            Some((&mut split.1[0], split.0))
        }
    }

    #[inline]
    fn split_mut<'a, P>(&'a mut self, pred: P) -> SplitMut<'a, T, P> where P: FnMut(&T) -> bool {
        SplitMut { v: self, pred: pred, finished: false }
    }

    #[inline]
    fn splitn_mut<'a, P>(&'a mut self, n: usize, pred: P) -> SplitNMut<'a, T, P> where
        P: FnMut(&T) -> bool
    {
        SplitNMut {
            inner: GenericSplitN {
                iter: self.split_mut(pred),
                count: n,
                invert: false
            }
        }
    }

    #[inline]
    fn rsplitn_mut<'a, P>(&'a mut self, n: usize, pred: P) -> RSplitNMut<'a, T, P> where
        P: FnMut(&T) -> bool,
    {
        RSplitNMut {
            inner: GenericSplitN {
                iter: self.split_mut(pred),
                count: n,
                invert: true
            }
        }
   }

    #[inline]
    fn chunks_mut(&mut self, chunk_size: usize) -> ChunksMut<T> {
        assert!(chunk_size > 0);
        ChunksMut { v: self, chunk_size: chunk_size }
    }

    #[inline]
    fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            // Can't take two mutable loans from one vector, so instead just cast
            // them to their raw pointers to do the swap
            let pa: *mut T = &mut self[a];
            let pb: *mut T = &mut self[b];
            ptr::swap(pa, pb);
        }
    }

    fn reverse(&mut self) {
        let mut i: usize = 0;
        let ln = self.len();
        while i < ln / 2 {
            // Unsafe swap to avoid the bounds check in safe swap.
            unsafe {
                let pa: *mut T = self.get_unchecked_mut(i);
                let pb: *mut T = self.get_unchecked_mut(ln - i - 1);
                ptr::swap(pa, pb);
            }
            i += 1;
        }
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        transmute((self.repr().data as *mut T).offset(index as isize))
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.repr().data as *mut T
    }

    #[inline]
    fn position_elem(&self, x: &T) -> Option<usize> where T: PartialEq {
        self.iter().position(|y| *x == *y)
    }

    #[inline]
    fn rposition_elem(&self, t: &T) -> Option<usize> where T: PartialEq {
        self.iter().rposition(|x| *x == *t)
    }

    #[inline]
    fn contains(&self, x: &T) -> bool where T: PartialEq {
        self.iter().any(|elt| *x == *elt)
    }

    #[inline]
    fn starts_with(&self, needle: &[T]) -> bool where T: PartialEq {
        let n = needle.len();
        self.len() >= n && needle == &self[..n]
    }

    #[inline]
    fn ends_with(&self, needle: &[T]) -> bool where T: PartialEq {
        let (m, n) = (self.len(), needle.len());
        m >= n && needle == &self[m-n..]
    }

    fn binary_search(&self, x: &T) -> Result<usize, usize> where T: Ord {
        self.binary_search_by(|p| p.cmp(x))
    }

    fn next_permutation(&mut self) -> bool where T: Ord {
        // These cases only have 1 permutation each, so we can't do anything.
        if self.len() < 2 { return false; }

        // Step 1: Identify the longest, rightmost weakly decreasing part of the vector
        let mut i = self.len() - 1;
        while i > 0 && self[i-1] >= self[i] {
            i -= 1;
        }

        // If that is the entire vector, this is the last-ordered permutation.
        if i == 0 {
            return false;
        }

        // Step 2: Find the rightmost element larger than the pivot (i-1)
        let mut j = self.len() - 1;
        while j >= i && self[j] <= self[i-1]  {
            j -= 1;
        }

        // Step 3: Swap that element with the pivot
        self.swap(j, i-1);

        // Step 4: Reverse the (previously) weakly decreasing part
        self[i..].reverse();

        true
    }

    fn prev_permutation(&mut self) -> bool where T: Ord {
        // These cases only have 1 permutation each, so we can't do anything.
        if self.len() < 2 { return false; }

        // Step 1: Identify the longest, rightmost weakly increasing part of the vector
        let mut i = self.len() - 1;
        while i > 0 && self[i-1] <= self[i] {
            i -= 1;
        }

        // If that is the entire vector, this is the first-ordered permutation.
        if i == 0 {
            return false;
        }

        // Step 2: Reverse the weakly increasing part
        self[i..].reverse();

        // Step 3: Find the rightmost element equal to or bigger than the pivot (i-1)
        let mut j = self.len() - 1;
        while j >= i && self[j-1] < self[i-1]  {
            j -= 1;
        }

        // Step 4: Swap that element with the pivot
        self.swap(i-1, j);

        true
    }

    #[inline]
    fn clone_from_slice(&mut self, src: &[T]) -> usize where T: Clone {
        let min = cmp::min(self.len(), src.len());
        let dst = &mut self[.. min];
        let src = &src[.. min];
        for i in 0..min {
            dst[i].clone_from(&src[i]);
        }
        min
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::Index<usize> for [T] {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        assert!(index < self.len());

        unsafe { mem::transmute(self.repr().data.offset(index as isize)) }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::IndexMut<usize> for [T] {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.len());

        unsafe { mem::transmute(self.repr().data.offset(index as isize)) }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::Index<ops::Range<usize>> for [T] {
    type Output = [T];

    #[inline]
    fn index(&self, index: ops::Range<usize>) -> &[T] {
        assert!(index.start <= index.end);
        assert!(index.end <= self.len());
        unsafe {
            from_raw_parts (
                self.as_ptr().offset(index.start as isize),
                index.end - index.start
            )
        }
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::Index<ops::RangeTo<usize>> for [T] {
    type Output = [T];

    #[inline]
    fn index(&self, index: ops::RangeTo<usize>) -> &[T] {
        self.index(ops::Range{ start: 0, end: index.end })
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::Index<ops::RangeFrom<usize>> for [T] {
    type Output = [T];

    #[inline]
    fn index(&self, index: ops::RangeFrom<usize>) -> &[T] {
        self.index(ops::Range{ start: index.start, end: self.len() })
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::Index<RangeFull> for [T] {
    type Output = [T];

    #[inline]
    fn index(&self, _index: RangeFull) -> &[T] {
        self
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::IndexMut<ops::Range<usize>> for [T] {
    #[inline]
    fn index_mut(&mut self, index: ops::Range<usize>) -> &mut [T] {
        assert!(index.start <= index.end);
        assert!(index.end <= self.len());
        unsafe {
            from_raw_parts_mut(
                self.as_mut_ptr().offset(index.start as isize),
                index.end - index.start
            )
        }
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::IndexMut<ops::RangeTo<usize>> for [T] {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeTo<usize>) -> &mut [T] {
        self.index_mut(ops::Range{ start: 0, end: index.end })
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::IndexMut<ops::RangeFrom<usize>> for [T] {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeFrom<usize>) -> &mut [T] {
        let len = self.len();
        self.index_mut(ops::Range{ start: index.start, end: len })
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::IndexMut<RangeFull> for [T] {
    #[inline]
    fn index_mut(&mut self, _index: RangeFull) -> &mut [T] {
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
// Common traits
////////////////////////////////////////////////////////////////////////////////

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Default for &'a [T] {
    #[stable(feature = "rust1", since = "1.0.0")]
    fn default() -> &'a [T] { &[] }
}

//
// Iterators
//

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> IntoIterator for &'a [T] {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> IntoIterator for &'a mut [T] {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

#[inline(always)]
fn size_from_ptr<T>(_: *const T) -> usize {
    mem::size_of::<T>()
}

// The shared definition of the `Iter` and `IterMut` iterators
macro_rules! iterator {
    (struct $name:ident -> $ptr:ty, $elem:ty) => {
        #[stable(feature = "rust1", since = "1.0.0")]
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = $elem;

            #[inline]
            fn next(&mut self) -> Option<$elem> {
                // could be implemented with slices, but this avoids bounds checks
                unsafe {
                    if mem::size_of::<T>() != 0 {
                        assume(!self.ptr.is_null());
                        assume(!self.end.is_null());
                    }
                    if self.ptr == self.end {
                        None
                    } else {
                        let old = self.ptr;
                        self.ptr = slice_offset!(self.ptr, 1);
                        Some(slice_ref!(old))
                    }
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let diff = (self.end as usize).wrapping_sub(self.ptr as usize);
                let size = mem::size_of::<T>();
                let exact = diff / (if size == 0 {1} else {size});
                (exact, Some(exact))
            }

            #[inline]
            fn count(self) -> usize {
                self.size_hint().0
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<$elem> {
                // Call helper method. Can't put the definition here because mut versus const.
                self.iter_nth(n)
            }

            #[inline]
            fn last(mut self) -> Option<$elem> {
                self.next_back()
            }
        }

        #[stable(feature = "rust1", since = "1.0.0")]
        impl<'a, T> DoubleEndedIterator for $name<'a, T> {
            #[inline]
            fn next_back(&mut self) -> Option<$elem> {
                // could be implemented with slices, but this avoids bounds checks
                unsafe {
                    if mem::size_of::<T>() != 0 {
                        assume(!self.ptr.is_null());
                        assume(!self.end.is_null());
                    }
                    if self.end == self.ptr {
                        None
                    } else {
                        self.end = slice_offset!(self.end, -1);
                        Some(slice_ref!(self.end))
                    }
                }
            }
        }
    }
}

macro_rules! make_slice {
    ($start: expr, $end: expr) => {{
        let start = $start;
        let diff = ($end as usize).wrapping_sub(start as usize);
        if size_from_ptr(start) == 0 {
            // use a non-null pointer value
            unsafe { from_raw_parts(1 as *const _, diff) }
        } else {
            let len = diff / size_from_ptr(start);
            unsafe { from_raw_parts(start, len) }
        }
    }}
}

macro_rules! make_mut_slice {
    ($start: expr, $end: expr) => {{
        let start = $start;
        let diff = ($end as usize).wrapping_sub(start as usize);
        if size_from_ptr(start) == 0 {
            // use a non-null pointer value
            unsafe { from_raw_parts_mut(1 as *mut _, diff) }
        } else {
            let len = diff / size_from_ptr(start);
            unsafe { from_raw_parts_mut(start, len) }
        }
    }}
}

/// Immutable slice iterator
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Iter<'a, T: 'a> {
    ptr: *const T,
    end: *const T,
    _marker: marker::PhantomData<&'a T>,
}

unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}
unsafe impl<'a, T: Sync> Send for Iter<'a, T> {}

impl<'a, T> Iter<'a, T> {
    /// View the underlying data as a subslice of the original data.
    ///
    /// This has the same lifetime as the original slice, and so the
    /// iterator can continue to be used while this exists.
    #[unstable(feature = "iter_to_slice")]
    pub fn as_slice(&self) -> &'a [T] {
        make_slice!(self.ptr, self.end)
    }

    // Helper function for Iter::nth
    fn iter_nth(&mut self, n: usize) -> Option<&'a T> {
        match self.as_slice().get(n) {
            Some(elem_ref) => unsafe {
                self.ptr = slice_offset!(self.ptr, (n as isize).wrapping_add(1));
                Some(elem_ref)
            },
            None => {
                self.ptr = self.end;
                None
            }
        }
    }
}

iterator!{struct Iter -> *const T, &'a T}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Iter<'a, T> { Iter { ptr: self.ptr, end: self.end, _marker: self._marker } }
}

#[unstable(feature = "iter_idx", reason = "trait is experimental")]
#[allow(deprecated)]
impl<'a, T> RandomAccessIterator for Iter<'a, T> {
    #[inline]
    fn indexable(&self) -> usize {
        let (exact, _) = self.size_hint();
        exact
    }

    #[inline]
    fn idx(&mut self, index: usize) -> Option<&'a T> {
        unsafe {
            if index < self.indexable() {
                Some(slice_ref!(self.ptr.offset(index as isize)))
            } else {
                None
            }
        }
    }
}

/// Mutable slice iterator.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct IterMut<'a, T: 'a> {
    ptr: *mut T,
    end: *mut T,
    _marker: marker::PhantomData<&'a mut T>,
}

unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}
unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}

impl<'a, T> IterMut<'a, T> {
    /// View the underlying data as a subslice of the original data.
    ///
    /// To avoid creating `&mut` references that alias, this is forced
    /// to consume the iterator. Consider using the `Slice` and
    /// `SliceMut` implementations for obtaining slices with more
    /// restricted lifetimes that do not consume the iterator.
    #[unstable(feature = "iter_to_slice")]
    pub fn into_slice(self) -> &'a mut [T] {
        make_mut_slice!(self.ptr, self.end)
    }

    // Helper function for IterMut::nth
    fn iter_nth(&mut self, n: usize) -> Option<&'a mut T> {
        match make_mut_slice!(self.ptr, self.end).get_mut(n) {
            Some(elem_ref) => unsafe {
                self.ptr = slice_offset!(self.ptr, (n as isize).wrapping_add(1));
                Some(elem_ref)
            },
            None => {
                self.ptr = self.end;
                None
            }
        }
    }
}

iterator!{struct IterMut -> *mut T, &'a mut T}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> ExactSizeIterator for IterMut<'a, T> {}

/// An internal abstraction over the splitting iterators, so that
/// splitn, splitn_mut etc can be implemented once.
trait SplitIter: DoubleEndedIterator {
    /// Mark the underlying iterator as complete, extracting the remaining
    /// portion of the slice.
    fn finish(&mut self) -> Option<Self::Item>;
}

/// An iterator over subslices separated by elements that match a predicate
/// function.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Split<'a, T:'a, P> where P: FnMut(&T) -> bool {
    v: &'a [T],
    pred: P,
    finished: bool
}

// FIXME(#19839) Remove in favor of `#[derive(Clone)]`
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T, P> Clone for Split<'a, T, P> where P: Clone + FnMut(&T) -> bool {
    fn clone(&self) -> Split<'a, T, P> {
        Split {
            v: self.v,
            pred: self.pred.clone(),
            finished: self.finished,
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T, P> Iterator for Split<'a, T, P> where P: FnMut(&T) -> bool {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<&'a [T]> {
        if self.finished { return None; }

        match self.v.iter().position(|x| (self.pred)(x)) {
            None => self.finish(),
            Some(idx) => {
                let ret = Some(&self.v[..idx]);
                self.v = &self.v[idx + 1..];
                ret
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.finished {
            (0, Some(0))
        } else {
            (1, Some(self.v.len() + 1))
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T, P> DoubleEndedIterator for Split<'a, T, P> where P: FnMut(&T) -> bool {
    #[inline]
    fn next_back(&mut self) -> Option<&'a [T]> {
        if self.finished { return None; }

        match self.v.iter().rposition(|x| (self.pred)(x)) {
            None => self.finish(),
            Some(idx) => {
                let ret = Some(&self.v[idx + 1..]);
                self.v = &self.v[..idx];
                ret
            }
        }
    }
}

impl<'a, T, P> SplitIter for Split<'a, T, P> where P: FnMut(&T) -> bool {
    #[inline]
    fn finish(&mut self) -> Option<&'a [T]> {
        if self.finished { None } else { self.finished = true; Some(self.v) }
    }
}

/// An iterator over the subslices of the vector which are separated
/// by elements that match `pred`.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct SplitMut<'a, T:'a, P> where P: FnMut(&T) -> bool {
    v: &'a mut [T],
    pred: P,
    finished: bool
}

impl<'a, T, P> SplitIter for SplitMut<'a, T, P> where P: FnMut(&T) -> bool {
    #[inline]
    fn finish(&mut self) -> Option<&'a mut [T]> {
        if self.finished {
            None
        } else {
            self.finished = true;
            Some(mem::replace(&mut self.v, &mut []))
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T, P> Iterator for SplitMut<'a, T, P> where P: FnMut(&T) -> bool {
    type Item = &'a mut [T];

    #[inline]
    fn next(&mut self) -> Option<&'a mut [T]> {
        if self.finished { return None; }

        let idx_opt = { // work around borrowck limitations
            let pred = &mut self.pred;
            self.v.iter().position(|x| (*pred)(x))
        };
        match idx_opt {
            None => self.finish(),
            Some(idx) => {
                let tmp = mem::replace(&mut self.v, &mut []);
                let (head, tail) = tmp.split_at_mut(idx);
                self.v = &mut tail[1..];
                Some(head)
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.finished {
            (0, Some(0))
        } else {
            // if the predicate doesn't match anything, we yield one slice
            // if it matches every element, we yield len+1 empty slices.
            (1, Some(self.v.len() + 1))
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T, P> DoubleEndedIterator for SplitMut<'a, T, P> where
    P: FnMut(&T) -> bool,
{
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut [T]> {
        if self.finished { return None; }

        let idx_opt = { // work around borrowck limitations
            let pred = &mut self.pred;
            self.v.iter().rposition(|x| (*pred)(x))
        };
        match idx_opt {
            None => self.finish(),
            Some(idx) => {
                let tmp = mem::replace(&mut self.v, &mut []);
                let (head, tail) = tmp.split_at_mut(idx);
                self.v = head;
                Some(&mut tail[1..])
            }
        }
    }
}

/// An private iterator over subslices separated by elements that
/// match a predicate function, splitting at most a fixed number of
/// times.
struct GenericSplitN<I> {
    iter: I,
    count: usize,
    invert: bool
}

impl<T, I: SplitIter<Item=T>> Iterator for GenericSplitN<I> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        match self.count {
            0 => None,
            1 => { self.count -= 1; self.iter.finish() }
            _ => {
                self.count -= 1;
                if self.invert {self.iter.next_back()} else {self.iter.next()}
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper_opt) = self.iter.size_hint();
        (lower, upper_opt.map(|upper| cmp::min(self.count, upper)))
    }
}

/// An iterator over subslices separated by elements that match a predicate
/// function, limited to a given number of splits.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct SplitN<'a, T: 'a, P> where P: FnMut(&T) -> bool {
    inner: GenericSplitN<Split<'a, T, P>>
}

/// An iterator over subslices separated by elements that match a
/// predicate function, limited to a given number of splits, starting
/// from the end of the slice.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct RSplitN<'a, T: 'a, P> where P: FnMut(&T) -> bool {
    inner: GenericSplitN<Split<'a, T, P>>
}

/// An iterator over subslices separated by elements that match a predicate
/// function, limited to a given number of splits.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct SplitNMut<'a, T: 'a, P> where P: FnMut(&T) -> bool {
    inner: GenericSplitN<SplitMut<'a, T, P>>
}

/// An iterator over subslices separated by elements that match a
/// predicate function, limited to a given number of splits, starting
/// from the end of the slice.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct RSplitNMut<'a, T: 'a, P> where P: FnMut(&T) -> bool {
    inner: GenericSplitN<SplitMut<'a, T, P>>
}

macro_rules! forward_iterator {
    ($name:ident: $elem:ident, $iter_of:ty) => {
        #[stable(feature = "rust1", since = "1.0.0")]
        impl<'a, $elem, P> Iterator for $name<'a, $elem, P> where
            P: FnMut(&T) -> bool
        {
            type Item = $iter_of;

            #[inline]
            fn next(&mut self) -> Option<$iter_of> {
                self.inner.next()
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }
    }
}

forward_iterator! { SplitN: T, &'a [T] }
forward_iterator! { RSplitN: T, &'a [T] }
forward_iterator! { SplitNMut: T, &'a mut [T] }
forward_iterator! { RSplitNMut: T, &'a mut [T] }

/// An iterator over overlapping subslices of length `size`.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Windows<'a, T:'a> {
    v: &'a [T],
    size: usize
}

// FIXME(#19839) Remove in favor of `#[derive(Clone)]`
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Clone for Windows<'a, T> {
    fn clone(&self) -> Windows<'a, T> {
        Windows {
            v: self.v,
            size: self.size,
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Iterator for Windows<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<&'a [T]> {
        if self.size > self.v.len() {
            None
        } else {
            let ret = Some(&self.v[..self.size]);
            self.v = &self.v[1..];
            ret
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.size > self.v.len() {
            (0, Some(0))
        } else {
            let size = self.v.len() - self.size + 1;
            (size, Some(size))
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> DoubleEndedIterator for Windows<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a [T]> {
        if self.size > self.v.len() {
            None
        } else {
            let ret = Some(&self.v[self.v.len()-self.size..]);
            self.v = &self.v[..self.v.len()-1];
            ret
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> ExactSizeIterator for Windows<'a, T> {}

#[unstable(feature = "iter_idx", reason = "trait is experimental")]
#[allow(deprecated)]
impl<'a, T> RandomAccessIterator for Windows<'a, T> {
    #[inline]
    fn indexable(&self) -> usize {
        self.size_hint().0
    }

    #[inline]
    fn idx(&mut self, index: usize) -> Option<&'a [T]> {
        if index + self.size > self.v.len() {
            None
        } else {
            Some(&self.v[index .. index+self.size])
        }
    }
}

/// An iterator over a slice in (non-overlapping) chunks (`size` elements at a
/// time).
///
/// When the slice len is not evenly divided by the chunk size, the last slice
/// of the iteration will be the remainder.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Chunks<'a, T:'a> {
    v: &'a [T],
    size: usize
}

// FIXME(#19839) Remove in favor of `#[derive(Clone)]`
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Clone for Chunks<'a, T> {
    fn clone(&self) -> Chunks<'a, T> {
        Chunks {
            v: self.v,
            size: self.size,
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Iterator for Chunks<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<&'a [T]> {
        if self.v.is_empty() {
            None
        } else {
            let chunksz = cmp::min(self.v.len(), self.size);
            let (fst, snd) = self.v.split_at(chunksz);
            self.v = snd;
            Some(fst)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.v.is_empty() {
            (0, Some(0))
        } else {
            let n = self.v.len() / self.size;
            let rem = self.v.len() % self.size;
            let n = if rem > 0 { n+1 } else { n };
            (n, Some(n))
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> DoubleEndedIterator for Chunks<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a [T]> {
        if self.v.is_empty() {
            None
        } else {
            let remainder = self.v.len() % self.size;
            let chunksz = if remainder != 0 { remainder } else { self.size };
            let (fst, snd) = self.v.split_at(self.v.len() - chunksz);
            self.v = fst;
            Some(snd)
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> ExactSizeIterator for Chunks<'a, T> {}

#[unstable(feature = "iter_idx", reason = "trait is experimental")]
#[allow(deprecated)]
impl<'a, T> RandomAccessIterator for Chunks<'a, T> {
    #[inline]
    fn indexable(&self) -> usize {
        self.v.len()/self.size + if self.v.len() % self.size != 0 { 1 } else { 0 }
    }

    #[inline]
    fn idx(&mut self, index: usize) -> Option<&'a [T]> {
        if index < self.indexable() {
            let lo = index * self.size;
            let mut hi = lo + self.size;
            if hi < lo || hi > self.v.len() { hi = self.v.len(); }

            Some(&self.v[lo..hi])
        } else {
            None
        }
    }
}

/// An iterator over a slice in (non-overlapping) mutable chunks (`size`
/// elements at a time). When the slice len is not evenly divided by the chunk
/// size, the last slice of the iteration will be the remainder.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct ChunksMut<'a, T:'a> {
    v: &'a mut [T],
    chunk_size: usize
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Iterator for ChunksMut<'a, T> {
    type Item = &'a mut [T];

    #[inline]
    fn next(&mut self) -> Option<&'a mut [T]> {
        if self.v.is_empty() {
            None
        } else {
            let sz = cmp::min(self.v.len(), self.chunk_size);
            let tmp = mem::replace(&mut self.v, &mut []);
            let (head, tail) = tmp.split_at_mut(sz);
            self.v = tail;
            Some(head)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.v.is_empty() {
            (0, Some(0))
        } else {
            let n = self.v.len() / self.chunk_size;
            let rem = self.v.len() % self.chunk_size;
            let n = if rem > 0 { n + 1 } else { n };
            (n, Some(n))
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> DoubleEndedIterator for ChunksMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut [T]> {
        if self.v.is_empty() {
            None
        } else {
            let remainder = self.v.len() % self.chunk_size;
            let sz = if remainder != 0 { remainder } else { self.chunk_size };
            let tmp = mem::replace(&mut self.v, &mut []);
            let tmp_len = tmp.len();
            let (head, tail) = tmp.split_at_mut(tmp_len - sz);
            self.v = head;
            Some(tail)
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> ExactSizeIterator for ChunksMut<'a, T> {}

//
// Free functions
//

/// Converts a pointer to A into a slice of length 1 (without copying).
#[unstable(feature = "ref_slice")]
pub fn ref_slice<'a, A>(s: &'a A) -> &'a [A] {
    unsafe {
        from_raw_parts(s, 1)
    }
}

/// Converts a pointer to A into a slice of length 1 (without copying).
#[unstable(feature = "ref_slice")]
pub fn mut_ref_slice<'a, A>(s: &'a mut A) -> &'a mut [A] {
    unsafe {
        from_raw_parts_mut(s, 1)
    }
}

/// Forms a slice from a pointer and a length.
///
/// The `len` argument is the number of **elements**, not the number of bytes.
///
/// # Unsafety
///
/// This function is unsafe as there is no guarantee that the given pointer is
/// valid for `len` elements, nor whether the lifetime inferred is a suitable
/// lifetime for the returned slice.
///
/// `p` must be non-null, even for zero-length slices.
///
/// # Caveat
///
/// The lifetime for the returned slice is inferred from its usage. To
/// prevent accidental misuse, it's suggested to tie the lifetime to whichever
/// source lifetime is safe in the context, such as by providing a helper
/// function taking the lifetime of a host value for the slice, or by explicit
/// annotation.
///
/// # Examples
///
/// ```
/// use std::slice;
///
/// // manifest a slice out of thin air!
/// let ptr = 0x1234 as *const usize;
/// let amt = 10;
/// unsafe {
///     let slice = slice::from_raw_parts(ptr, amt);
/// }
/// ```
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn from_raw_parts<'a, T>(p: *const T, len: usize) -> &'a [T] {
    transmute(RawSlice { data: p, len: len })
}

/// Performs the same functionality as `from_raw_parts`, except that a mutable
/// slice is returned.
///
/// This function is unsafe for the same reasons as `from_raw_parts`, as well
/// as not being able to provide a non-aliasing guarantee of the returned
/// mutable slice.
#[inline]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn from_raw_parts_mut<'a, T>(p: *mut T, len: usize) -> &'a mut [T] {
    transmute(RawSlice { data: p, len: len })
}

//
// Submodules
//

/// Operations on `[u8]`.
#[unstable(feature = "slice_bytes", reason = "needs review")]
pub mod bytes {
    use ptr;
    use slice::SliceExt;

    /// A trait for operations on mutable `[u8]`s.
    pub trait MutableByteVector {
        /// Sets all bytes of the receiver to the given value.
        fn set_memory(&mut self, value: u8);
    }

    impl MutableByteVector for [u8] {
        #[inline]
        fn set_memory(&mut self, value: u8) {
            unsafe { ptr::write_bytes(self.as_mut_ptr(), value, self.len()) };
        }
    }

    /// Copies data from `src` to `dst`
    ///
    /// Panics if the length of `dst` is less than the length of `src`.
    #[inline]
    pub fn copy_memory(src: &[u8], dst: &mut [u8]) {
        let len_src = src.len();
        assert!(dst.len() >= len_src);
        // `dst` is unaliasable, so we know statically it doesn't overlap
        // with `src`.
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(),
                                     dst.as_mut_ptr(),
                                     len_src);
        }
    }
}



//
// Boilerplate traits
//

#[stable(feature = "rust1", since = "1.0.0")]
impl<A, B> PartialEq<[B]> for [A] where A: PartialEq<B> {
    fn eq(&self, other: &[B]) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for i in 0..self.len() {
            if !self[i].eq(&other[i]) {
                return false;
            }
        }

        true
    }
    fn ne(&self, other: &[B]) -> bool {
        if self.len() != other.len() {
            return true;
        }

        for i in 0..self.len() {
            if self[i].ne(&other[i]) {
                return true;
            }
        }

        false
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: Eq> Eq for [T] {}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: Ord> Ord for [T] {
    fn cmp(&self, other: &[T]) -> Ordering {
        order::cmp(self.iter(), other.iter())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: PartialOrd> PartialOrd for [T] {
    #[inline]
    fn partial_cmp(&self, other: &[T]) -> Option<Ordering> {
        order::partial_cmp(self.iter(), other.iter())
    }
    #[inline]
    fn lt(&self, other: &[T]) -> bool {
        order::lt(self.iter(), other.iter())
    }
    #[inline]
    fn le(&self, other: &[T]) -> bool {
        order::le(self.iter(), other.iter())
    }
    #[inline]
    fn ge(&self, other: &[T]) -> bool {
        order::ge(self.iter(), other.iter())
    }
    #[inline]
    fn gt(&self, other: &[T]) -> bool {
        order::gt(self.iter(), other.iter())
    }
}

/// Extension methods for slices containing integers.
#[unstable(feature = "int_slice")]
#[deprecated(since = "1.2.0",
             reason = "has not seen much usage and may want to live in the \
                       standard library now that most slice methods are \
                       on an inherent implementation block")]
pub trait IntSliceExt<U, S> {
    /// Converts the slice to an immutable slice of unsigned integers with the same width.
    fn as_unsigned<'a>(&'a self) -> &'a [U];
    /// Converts the slice to an immutable slice of signed integers with the same width.
    fn as_signed<'a>(&'a self) -> &'a [S];

    /// Converts the slice to a mutable slice of unsigned integers with the same width.
    fn as_unsigned_mut<'a>(&'a mut self) -> &'a mut [U];
    /// Converts the slice to a mutable slice of signed integers with the same width.
    fn as_signed_mut<'a>(&'a mut self) -> &'a mut [S];
}

macro_rules! impl_int_slice {
    ($u:ty, $s:ty, $t:ty) => {
        #[unstable(feature = "int_slice")]
        #[allow(deprecated)]
        impl IntSliceExt<$u, $s> for [$t] {
            #[inline]
            fn as_unsigned(&self) -> &[$u] { unsafe { transmute(self) } }
            #[inline]
            fn as_signed(&self) -> &[$s] { unsafe { transmute(self) } }
            #[inline]
            fn as_unsigned_mut(&mut self) -> &mut [$u] { unsafe { transmute(self) } }
            #[inline]
            fn as_signed_mut(&mut self) -> &mut [$s] { unsafe { transmute(self) } }
        }
    }
}

macro_rules! impl_int_slices {
    ($u:ty, $s:ty) => {
        impl_int_slice! { $u, $s, $u }
        impl_int_slice! { $u, $s, $s }
    }
}

impl_int_slices! { u8,   i8  }
impl_int_slices! { u16,  i16 }
impl_int_slices! { u32,  i32 }
impl_int_slices! { u64,  i64 }
impl_int_slices! { usize, isize }
