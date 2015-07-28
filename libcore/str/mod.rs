// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! String manipulation
//!
//! For more details, see std::str

#![stable(feature = "rust1", since = "1.0.0")]

use self::pattern::Pattern;
use self::pattern::{Searcher, ReverseSearcher, DoubleEndedSearcher};

use char::CharExt;
use clone::Clone;
use cmp::Eq;
use convert::AsRef;
use default::Default;
use fmt;
use iter::ExactSizeIterator;
use iter::{Map, Iterator, DoubleEndedIterator};
use mem;
use ops::{Fn, FnMut, FnOnce};
use option::Option::{self, None, Some};
use raw::{Repr, Slice};
use result::Result::{self, Ok, Err};
use slice::{self, SliceExt};

pub mod pattern;

/// A trait to abstract the idea of creating a new instance of a type from a
/// string.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait FromStr {
    /// The associated error which can be returned from parsing.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Err;

    /// Parses a string `s` to return a value of this type.
    ///
    /// If parsing succeeds, return the value inside `Ok`, otherwise
    /// when the string is ill-formatted return an error specific to the
    /// inside `Err`. The error type is specific to implementation of the trait.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl FromStr for bool {
    type Err = ParseBoolError;

    /// Parse a `bool` from a string.
    ///
    /// Yields a `Result<bool, ParseBoolError>`, because `s` may or may not
    /// actually be parseable.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// assert_eq!(FromStr::from_str("true"), Ok(true));
    /// assert_eq!(FromStr::from_str("false"), Ok(false));
    /// assert!(<bool as FromStr>::from_str("not even a boolean").is_err());
    /// ```
    ///
    /// Note, in many cases, the `.parse()` method on `str` is more proper.
    ///
    /// ```
    /// assert_eq!("true".parse(), Ok(true));
    /// assert_eq!("false".parse(), Ok(false));
    /// assert!("not even a boolean".parse::<bool>().is_err());
    /// ```
    #[inline]
    fn from_str(s: &str) -> Result<bool, ParseBoolError> {
        match s {
            "true"  => Ok(true),
            "false" => Ok(false),
            _       => Err(ParseBoolError { _priv: () }),
        }
    }
}

/// An error returned when parsing a `bool` from a string fails.
#[derive(Debug, Clone, PartialEq)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct ParseBoolError { _priv: () }

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Display for ParseBoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "provided string was not `true` or `false`".fmt(f)
    }
}

/*
Section: Creating a string
*/

/// Errors which can occur when attempting to interpret a byte slice as a `str`.
#[derive(Copy, Eq, PartialEq, Clone, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Utf8Error {
    valid_up_to: usize,
}

impl Utf8Error {
    /// Returns the index in the given string up to which valid UTF-8 was
    /// verified.
    ///
    /// Starting at the index provided, but not necessarily at it precisely, an
    /// invalid UTF-8 encoding sequence was found.
    #[unstable(feature = "utf8_error", reason = "method just added")]
    pub fn valid_up_to(&self) -> usize { self.valid_up_to }
}

/// Converts a slice of bytes to a string slice without performing any
/// allocations.
///
/// Once the slice has been validated as utf-8, it is transmuted in-place and
/// returned as a '&str' instead of a '&[u8]'
///
/// # Failure
///
/// Returns `Err` if the slice is not utf-8 with a description as to why the
/// provided slice is not utf-8.
#[stable(feature = "rust1", since = "1.0.0")]
pub fn from_utf8(v: &[u8]) -> Result<&str, Utf8Error> {
    try!(run_utf8_validation_iterator(&mut v.iter()));
    Ok(unsafe { from_utf8_unchecked(v) })
}

/// Converts a slice of bytes to a string slice without checking
/// that the string contains valid UTF-8.
#[inline(always)]
#[stable(feature = "rust1", since = "1.0.0")]
pub unsafe fn from_utf8_unchecked<'a>(v: &'a [u8]) -> &'a str {
    mem::transmute(v)
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Display for Utf8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid utf-8: invalid byte near index {}", self.valid_up_to)
    }
}

/*
Section: Iterators
*/

/// Iterator for the char (representing *Unicode Scalar Values*) of a string
///
/// Created with the method `.chars()`.
#[derive(Clone)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Chars<'a> {
    iter: slice::Iter<'a, u8>
}

/// Return the initial codepoint accumulator for the first byte.
/// The first byte is special, only want bottom 5 bits for width 2, 4 bits
/// for width 3, and 3 bits for width 4.
#[inline]
fn utf8_first_byte(byte: u8, width: u32) -> u32 { (byte & (0x7F >> width)) as u32 }

/// Return the value of `ch` updated with continuation byte `byte`.
#[inline]
fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 { (ch << 6) | (byte & CONT_MASK) as u32 }

/// Checks whether the byte is a UTF-8 continuation byte (i.e. starts with the
/// bits `10`).
#[inline]
fn utf8_is_cont_byte(byte: u8) -> bool { (byte & !CONT_MASK) == TAG_CONT_U8 }

#[inline]
fn unwrap_or_0(opt: Option<&u8>) -> u8 {
    match opt {
        Some(&byte) => byte,
        None => 0,
    }
}

/// Reads the next code point out of a byte iterator (assuming a
/// UTF-8-like encoding).
#[unstable(feature = "str_internals")]
#[inline]
pub fn next_code_point(bytes: &mut slice::Iter<u8>) -> Option<u32> {
    // Decode UTF-8
    let x = match bytes.next() {
        None => return None,
        Some(&next_byte) if next_byte < 128 => return Some(next_byte as u32),
        Some(&next_byte) => next_byte,
    };

    // Multibyte case follows
    // Decode from a byte combination out of: [[[x y] z] w]
    // NOTE: Performance is sensitive to the exact formulation here
    let init = utf8_first_byte(x, 2);
    let y = unwrap_or_0(bytes.next());
    let mut ch = utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        // [[x y z] w] case
        // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
        let z = unwrap_or_0(bytes.next());
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        ch = init << 12 | y_z;
        if x >= 0xF0 {
            // [x y z w] case
            // use only the lower 3 bits of `init`
            let w = unwrap_or_0(bytes.next());
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    Some(ch)
}

/// Reads the last code point out of a byte iterator (assuming a
/// UTF-8-like encoding).
#[inline]
fn next_code_point_reverse(bytes: &mut slice::Iter<u8>) -> Option<u32> {
    // Decode UTF-8
    let w = match bytes.next_back() {
        None => return None,
        Some(&next_byte) if next_byte < 128 => return Some(next_byte as u32),
        Some(&back_byte) => back_byte,
    };

    // Multibyte case follows
    // Decode from a byte combination out of: [x [y [z w]]]
    let mut ch;
    let z = unwrap_or_0(bytes.next_back());
    ch = utf8_first_byte(z, 2);
    if utf8_is_cont_byte(z) {
        let y = unwrap_or_0(bytes.next_back());
        ch = utf8_first_byte(y, 3);
        if utf8_is_cont_byte(y) {
            let x = unwrap_or_0(bytes.next_back());
            ch = utf8_first_byte(x, 4);
            ch = utf8_acc_cont_byte(ch, y);
        }
        ch = utf8_acc_cont_byte(ch, z);
    }
    ch = utf8_acc_cont_byte(ch, w);

    Some(ch)
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for Chars<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        next_code_point(&mut self.iter).map(|ch| {
            // str invariant says `ch` is a valid Unicode Scalar Value
            unsafe {
                mem::transmute(ch)
            }
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (len, _) = self.iter.size_hint();
        // `(len + 3)` can't overflow, because we know that the `slice::Iter`
        // belongs to a slice in memory which has a maximum length of
        // `isize::MAX` (that's well below `usize::MAX`).
        ((len + 3) / 4, Some(len))
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> DoubleEndedIterator for Chars<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<char> {
        next_code_point_reverse(&mut self.iter).map(|ch| {
            // str invariant says `ch` is a valid Unicode Scalar Value
            unsafe {
                mem::transmute(ch)
            }
        })
    }
}

/// Iterator for a string's characters and their byte offsets.
#[derive(Clone)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct CharIndices<'a> {
    front_offset: usize,
    iter: Chars<'a>,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for CharIndices<'a> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<(usize, char)> {
        let (pre_len, _) = self.iter.iter.size_hint();
        match self.iter.next() {
            None => None,
            Some(ch) => {
                let index = self.front_offset;
                let (len, _) = self.iter.iter.size_hint();
                self.front_offset += pre_len - len;
                Some((index, ch))
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> DoubleEndedIterator for CharIndices<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<(usize, char)> {
        match self.iter.next_back() {
            None => None,
            Some(ch) => {
                let (len, _) = self.iter.iter.size_hint();
                let index = self.front_offset + len;
                Some((index, ch))
            }
        }
    }
}

/// External iterator for a string's bytes.
/// Use with the `std::iter` module.
///
/// Created with the method `.bytes()`.
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Clone)]
pub struct Bytes<'a>(Map<slice::Iter<'a, u8>, BytesDeref>);

/// A nameable, clonable fn type
#[derive(Clone)]
struct BytesDeref;

impl<'a> Fn<(&'a u8,)> for BytesDeref {
    #[inline]
    extern "rust-call" fn call(&self, (ptr,): (&'a u8,)) -> u8 {
        *ptr
    }
}

impl<'a> FnMut<(&'a u8,)> for BytesDeref {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, (ptr,): (&'a u8,)) -> u8 {
        Fn::call(&*self, (ptr,))
    }
}

impl<'a> FnOnce<(&'a u8,)> for BytesDeref {
    type Output = u8;

    #[inline]
    extern "rust-call" fn call_once(self, (ptr,): (&'a u8,)) -> u8 {
        Fn::call(&self, (ptr,))
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for Bytes<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> DoubleEndedIterator for Bytes<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<u8> {
        self.0.next_back()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> ExactSizeIterator for Bytes<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// This macro generates a Clone impl for string pattern API
/// wrapper types of the form X<'a, P>
macro_rules! derive_pattern_clone {
    (clone $t:ident with |$s:ident| $e:expr) => {
        impl<'a, P: Pattern<'a>> Clone for $t<'a, P>
            where P::Searcher: Clone
        {
            fn clone(&self) -> Self {
                let $s = self;
                $e
            }
        }
    }
}

/// This macro generates two public iterator structs
/// wrapping an private internal one that makes use of the `Pattern` API.
///
/// For all patterns `P: Pattern<'a>` the following items will be
/// generated (generics omitted):
///
/// struct $forward_iterator($internal_iterator);
/// struct $reverse_iterator($internal_iterator);
///
/// impl Iterator for $forward_iterator
/// { /* internal ends up calling Searcher::next_match() */ }
///
/// impl DoubleEndedIterator for $forward_iterator
///       where P::Searcher: DoubleEndedSearcher
/// { /* internal ends up calling Searcher::next_match_back() */ }
///
/// impl Iterator for $reverse_iterator
///       where P::Searcher: ReverseSearcher
/// { /* internal ends up calling Searcher::next_match_back() */ }
///
/// impl DoubleEndedIterator for $reverse_iterator
///       where P::Searcher: DoubleEndedSearcher
/// { /* internal ends up calling Searcher::next_match() */ }
///
/// The internal one is defined outside the macro, and has almost the same
/// semantic as a DoubleEndedIterator by delegating to `pattern::Searcher` and
/// `pattern::ReverseSearcher` for both forward and reverse iteration.
///
/// "Almost", because a `Searcher` and a `ReverseSearcher` for a given
/// `Pattern` might not return the same elements, so actually implementing
/// `DoubleEndedIterator` for it would be incorrect.
/// (See the docs in `str::pattern` for more details)
///
/// However, the internal struct still represents a single ended iterator from
/// either end, and depending on pattern is also a valid double ended iterator,
/// so the two wrapper structs implement `Iterator`
/// and `DoubleEndedIterator` depending on the concrete pattern type, leading
/// to the complex impls seen above.
macro_rules! generate_pattern_iterators {
    {
        // Forward iterator
        forward:
            $(#[$forward_iterator_attribute:meta])*
            struct $forward_iterator:ident;

        // Reverse iterator
        reverse:
            $(#[$reverse_iterator_attribute:meta])*
            struct $reverse_iterator:ident;

        // Stability of all generated items
        stability:
            $(#[$common_stability_attribute:meta])*

        // Internal almost-iterator that is being delegated to
        internal:
            $internal_iterator:ident yielding ($iterty:ty);

        // Kind of delgation - either single ended or double ended
        delegate $($t:tt)*
    } => {
        $(#[$forward_iterator_attribute])*
        $(#[$common_stability_attribute])*
        pub struct $forward_iterator<'a, P: Pattern<'a>>($internal_iterator<'a, P>);

        $(#[$common_stability_attribute])*
        impl<'a, P: Pattern<'a>> Iterator for $forward_iterator<'a, P> {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<$iterty> {
                self.0.next()
            }
        }

        $(#[$common_stability_attribute])*
        impl<'a, P: Pattern<'a>> Clone for $forward_iterator<'a, P>
            where P::Searcher: Clone
        {
            fn clone(&self) -> Self {
                $forward_iterator(self.0.clone())
            }
        }

        $(#[$reverse_iterator_attribute])*
        $(#[$common_stability_attribute])*
        pub struct $reverse_iterator<'a, P: Pattern<'a>>($internal_iterator<'a, P>);

        $(#[$common_stability_attribute])*
        impl<'a, P: Pattern<'a>> Iterator for $reverse_iterator<'a, P>
            where P::Searcher: ReverseSearcher<'a>
        {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<$iterty> {
                self.0.next_back()
            }
        }

        $(#[$common_stability_attribute])*
        impl<'a, P: Pattern<'a>> Clone for $reverse_iterator<'a, P>
            where P::Searcher: Clone
        {
            fn clone(&self) -> Self {
                $reverse_iterator(self.0.clone())
            }
        }

        generate_pattern_iterators!($($t)* with $(#[$common_stability_attribute])*,
                                                $forward_iterator,
                                                $reverse_iterator, $iterty);
    };
    {
        double ended; with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident, $iterty:ty
    } => {
        $(#[$common_stability_attribute])*
        impl<'a, P: Pattern<'a>> DoubleEndedIterator for $forward_iterator<'a, P>
            where P::Searcher: DoubleEndedSearcher<'a>
        {
            #[inline]
            fn next_back(&mut self) -> Option<$iterty> {
                self.0.next_back()
            }
        }

        $(#[$common_stability_attribute])*
        impl<'a, P: Pattern<'a>> DoubleEndedIterator for $reverse_iterator<'a, P>
            where P::Searcher: DoubleEndedSearcher<'a>
        {
            #[inline]
            fn next_back(&mut self) -> Option<$iterty> {
                self.0.next()
            }
        }
    };
    {
        single ended; with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident, $iterty:ty
    } => {}
}

derive_pattern_clone!{
    clone SplitInternal
    with |s| SplitInternal { matcher: s.matcher.clone(), ..*s }
}
struct SplitInternal<'a, P: Pattern<'a>> {
    start: usize,
    end: usize,
    matcher: P::Searcher,
    allow_trailing_empty: bool,
    finished: bool,
}

impl<'a, P: Pattern<'a>> SplitInternal<'a, P> {
    #[inline]
    fn get_end(&mut self) -> Option<&'a str> {
        if !self.finished && (self.allow_trailing_empty || self.end - self.start > 0) {
            self.finished = true;
            unsafe {
                let string = self.matcher.haystack().slice_unchecked(self.start, self.end);
                Some(string)
            }
        } else {
            None
        }
    }

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        if self.finished { return None }

        let haystack = self.matcher.haystack();
        match self.matcher.next_match() {
            Some((a, b)) => unsafe {
                let elt = haystack.slice_unchecked(self.start, a);
                self.start = b;
                Some(elt)
            },
            None => self.get_end(),
        }
    }

    #[inline]
    fn next_back(&mut self) -> Option<&'a str>
        where P::Searcher: ReverseSearcher<'a>
    {
        if self.finished { return None }

        if !self.allow_trailing_empty {
            self.allow_trailing_empty = true;
            match self.next_back() {
                Some(elt) if !elt.is_empty() => return Some(elt),
                _ => if self.finished { return None }
            }
        }

        let haystack = self.matcher.haystack();
        match self.matcher.next_match_back() {
            Some((a, b)) => unsafe {
                let elt = haystack.slice_unchecked(b, self.end);
                self.end = a;
                Some(elt)
            },
            None => unsafe {
                self.finished = true;
                Some(haystack.slice_unchecked(self.start, self.end))
            },
        }
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method `.split()`.
        struct Split;
    reverse:
        /// Created with the method `.rsplit()`.
        struct RSplit;
    stability:
        #[stable(feature = "rust1", since = "1.0.0")]
    internal:
        SplitInternal yielding (&'a str);
    delegate double ended;
}

generate_pattern_iterators! {
    forward:
        /// Created with the method `.split_terminator()`.
        struct SplitTerminator;
    reverse:
        /// Created with the method `.rsplit_terminator()`.
        struct RSplitTerminator;
    stability:
        #[stable(feature = "rust1", since = "1.0.0")]
    internal:
        SplitInternal yielding (&'a str);
    delegate double ended;
}

derive_pattern_clone!{
    clone SplitNInternal
    with |s| SplitNInternal { iter: s.iter.clone(), ..*s }
}
struct SplitNInternal<'a, P: Pattern<'a>> {
    iter: SplitInternal<'a, P>,
    /// The number of splits remaining
    count: usize,
}

impl<'a, P: Pattern<'a>> SplitNInternal<'a, P> {
    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        match self.count {
            0 => None,
            1 => { self.count = 0; self.iter.get_end() }
            _ => { self.count -= 1; self.iter.next() }
        }
    }

    #[inline]
    fn next_back(&mut self) -> Option<&'a str>
        where P::Searcher: ReverseSearcher<'a>
    {
        match self.count {
            0 => None,
            1 => { self.count = 0; self.iter.get_end() }
            _ => { self.count -= 1; self.iter.next_back() }
        }
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method `.splitn()`.
        struct SplitN;
    reverse:
        /// Created with the method `.rsplitn()`.
        struct RSplitN;
    stability:
        #[stable(feature = "rust1", since = "1.0.0")]
    internal:
        SplitNInternal yielding (&'a str);
    delegate single ended;
}

derive_pattern_clone!{
    clone MatchIndicesInternal
    with |s| MatchIndicesInternal(s.0.clone())
}
struct MatchIndicesInternal<'a, P: Pattern<'a>>(P::Searcher);

impl<'a, P: Pattern<'a>> MatchIndicesInternal<'a, P> {
    #[inline]
    fn next(&mut self) -> Option<(usize, usize)> {
        self.0.next_match()
    }

    #[inline]
    fn next_back(&mut self) -> Option<(usize, usize)>
        where P::Searcher: ReverseSearcher<'a>
    {
        self.0.next_match_back()
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method `.match_indices()`.
        struct MatchIndices;
    reverse:
        /// Created with the method `.rmatch_indices()`.
        struct RMatchIndices;
    stability:
        #[unstable(feature = "str_match_indices",
                   reason = "type may be removed or have its iterator impl changed")]
    internal:
        MatchIndicesInternal yielding ((usize, usize));
    delegate double ended;
}

derive_pattern_clone!{
    clone MatchesInternal
    with |s| MatchesInternal(s.0.clone())
}
struct MatchesInternal<'a, P: Pattern<'a>>(P::Searcher);

impl<'a, P: Pattern<'a>> MatchesInternal<'a, P> {
    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.0.next_match().map(|(a, b)| unsafe {
            // Indices are known to be on utf8 boundaries
            self.0.haystack().slice_unchecked(a, b)
        })
    }

    #[inline]
    fn next_back(&mut self) -> Option<&'a str>
        where P::Searcher: ReverseSearcher<'a>
    {
        self.0.next_match_back().map(|(a, b)| unsafe {
            // Indices are known to be on utf8 boundaries
            self.0.haystack().slice_unchecked(a, b)
        })
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method `.matches()`.
        struct Matches;
    reverse:
        /// Created with the method `.rmatches()`.
        struct RMatches;
    stability:
        #[stable(feature = "str_matches", since = "1.2.0")]
    internal:
        MatchesInternal yielding (&'a str);
    delegate double ended;
}

/// Created with the method `.lines()`.
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Clone)]
pub struct Lines<'a>(SplitTerminator<'a, char>);

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> DoubleEndedIterator for Lines<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a str> {
        self.0.next_back()
    }
}

/// Created with the method `.lines_any()`.
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Clone)]
pub struct LinesAny<'a>(Map<Lines<'a>, LinesAnyMap>);

/// A nameable, clonable fn type
#[derive(Clone)]
struct LinesAnyMap;

impl<'a> Fn<(&'a str,)> for LinesAnyMap {
    #[inline]
    extern "rust-call" fn call(&self, (line,): (&'a str,)) -> &'a str {
        let l = line.len();
        if l > 0 && line.as_bytes()[l - 1] == b'\r' { &line[0 .. l - 1] }
        else { line }
    }
}

impl<'a> FnMut<(&'a str,)> for LinesAnyMap {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, (line,): (&'a str,)) -> &'a str {
        Fn::call(&*self, (line,))
    }
}

impl<'a> FnOnce<(&'a str,)> for LinesAnyMap {
    type Output = &'a str;

    #[inline]
    extern "rust-call" fn call_once(self, (line,): (&'a str,)) -> &'a str {
        Fn::call(&self, (line,))
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for LinesAny<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> DoubleEndedIterator for LinesAny<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a str> {
        self.0.next_back()
    }
}

/*
Section: Comparing strings
*/

/// Bytewise slice equality
/// NOTE: This function is (ab)used in rustc::middle::trans::_match
/// to compare &[u8] byte slices that are not necessarily valid UTF-8.
#[lang = "str_eq"]
#[inline]
fn eq_slice(a: &str, b: &str) -> bool {
    // NOTE: In theory n should be libc::size_t and not usize, but libc is not available here
    #[allow(improper_ctypes)]
    extern { fn memcmp(s1: *const i8, s2: *const i8, n: usize) -> i32; }
    a.len() == b.len() && unsafe {
        memcmp(a.as_ptr() as *const i8,
               b.as_ptr() as *const i8,
               a.len()) == 0
    }
}

/*
Section: Misc
*/

/// Walk through `iter` checking that it's a valid UTF-8 sequence,
/// returning `true` in that case, or, if it is invalid, `false` with
/// `iter` reset such that it is pointing at the first byte in the
/// invalid sequence.
#[inline(always)]
fn run_utf8_validation_iterator(iter: &mut slice::Iter<u8>)
                                -> Result<(), Utf8Error> {
    let whole = iter.as_slice();
    loop {
        // save the current thing we're pointing at.
        let old = iter.clone();

        // restore the iterator we had at the start of this codepoint.
        macro_rules! err { () => {{
            *iter = old.clone();
            return Err(Utf8Error {
                valid_up_to: whole.len() - iter.as_slice().len()
            })
        }}}

        macro_rules! next { () => {
            match iter.next() {
                Some(a) => *a,
                // we needed data, but there was none: error!
                None => err!(),
            }
        }}

        let first = match iter.next() {
            Some(&b) => b,
            // we're at the end of the iterator and a codepoint
            // boundary at the same time, so this string is valid.
            None => return Ok(())
        };

        // ASCII characters are always valid, so only large
        // bytes need more examination.
        if first >= 128 {
            let w = UTF8_CHAR_WIDTH[first as usize];
            let second = next!();
            // 2-byte encoding is for codepoints  \u{0080} to  \u{07ff}
            //        first  C2 80        last DF BF
            // 3-byte encoding is for codepoints  \u{0800} to  \u{ffff}
            //        first  E0 A0 80     last EF BF BF
            //   excluding surrogates codepoints  \u{d800} to  \u{dfff}
            //               ED A0 80 to       ED BF BF
            // 4-byte encoding is for codepoints \u{1000}0 to \u{10ff}ff
            //        first  F0 90 80 80  last F4 8F BF BF
            //
            // Use the UTF-8 syntax from the RFC
            //
            // https://tools.ietf.org/html/rfc3629
            // UTF8-1      = %x00-7F
            // UTF8-2      = %xC2-DF UTF8-tail
            // UTF8-3      = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2( UTF8-tail ) /
            //               %xED %x80-9F UTF8-tail / %xEE-EF 2( UTF8-tail )
            // UTF8-4      = %xF0 %x90-BF 2( UTF8-tail ) / %xF1-F3 3( UTF8-tail ) /
            //               %xF4 %x80-8F 2( UTF8-tail )
            match w {
                2 => if second & !CONT_MASK != TAG_CONT_U8 {err!()},
                3 => {
                    match (first, second, next!() & !CONT_MASK) {
                        (0xE0         , 0xA0 ... 0xBF, TAG_CONT_U8) |
                        (0xE1 ... 0xEC, 0x80 ... 0xBF, TAG_CONT_U8) |
                        (0xED         , 0x80 ... 0x9F, TAG_CONT_U8) |
                        (0xEE ... 0xEF, 0x80 ... 0xBF, TAG_CONT_U8) => {}
                        _ => err!()
                    }
                }
                4 => {
                    match (first, second, next!() & !CONT_MASK, next!() & !CONT_MASK) {
                        (0xF0         , 0x90 ... 0xBF, TAG_CONT_U8, TAG_CONT_U8) |
                        (0xF1 ... 0xF3, 0x80 ... 0xBF, TAG_CONT_U8, TAG_CONT_U8) |
                        (0xF4         , 0x80 ... 0x8F, TAG_CONT_U8, TAG_CONT_U8) => {}
                        _ => err!()
                    }
                }
                _ => err!()
            }
        }
    }
}

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

/// Struct that contains a `char` and the index of the first byte of
/// the next `char` in a string.  This can be used as a data structure
/// for iterating over the UTF-8 bytes of a string.
#[derive(Copy, Clone)]
#[unstable(feature = "str_char",
           reason = "existence of this struct is uncertain as it is frequently \
                     able to be replaced with char.len_utf8() and/or \
                     char/char_indices iterators")]
pub struct CharRange {
    /// Current `char`
    pub ch: char,
    /// Index of the first byte of the next `char`
    pub next: usize,
}

/// Mask of the value bits of a continuation byte
const CONT_MASK: u8 = 0b0011_1111;
/// Value of the tag bits (tag mask is !CONT_MASK) of a continuation byte
const TAG_CONT_U8: u8 = 0b1000_0000;

/*
Section: Trait implementations
*/

mod traits {
    use cmp::{Ordering, Ord, PartialEq, PartialOrd, Eq};
    use cmp::Ordering::{Less, Equal, Greater};
    use iter::Iterator;
    use option::Option;
    use option::Option::Some;
    use ops;
    use str::{StrExt, eq_slice};

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Ord for str {
        #[inline]
        fn cmp(&self, other: &str) -> Ordering {
            for (s_b, o_b) in self.bytes().zip(other.bytes()) {
                match s_b.cmp(&o_b) {
                    Greater => return Greater,
                    Less => return Less,
                    Equal => ()
                }
            }

            self.len().cmp(&other.len())
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl PartialEq for str {
        #[inline]
        fn eq(&self, other: &str) -> bool {
            eq_slice(self, other)
        }
        #[inline]
        fn ne(&self, other: &str) -> bool { !(*self).eq(other) }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Eq for str {}

    #[stable(feature = "rust1", since = "1.0.0")]
    impl PartialOrd for str {
        #[inline]
        fn partial_cmp(&self, other: &str) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    /// Returns a slice of the given string from the byte range
    /// [`begin`..`end`).
    ///
    /// This operation is `O(1)`.
    ///
    /// Panics when `begin` and `end` do not point to valid characters
    /// or point beyond the last character of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// let s = "Löwe 老虎 Léopard";
    /// assert_eq!(&s[0 .. 1], "L");
    ///
    /// assert_eq!(&s[1 .. 9], "öwe 老");
    ///
    /// // these will panic:
    /// // byte 2 lies within `ö`:
    /// // &s[2 ..3];
    ///
    /// // byte 8 lies within `老`
    /// // &s[1 .. 8];
    ///
    /// // byte 100 is outside the string
    /// // &s[3 .. 100];
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    impl ops::Index<ops::Range<usize>> for str {
        type Output = str;
        #[inline]
        fn index(&self, index: ops::Range<usize>) -> &str {
            // is_char_boundary checks that the index is in [0, .len()]
            if index.start <= index.end &&
               self.is_char_boundary(index.start) &&
               self.is_char_boundary(index.end) {
                unsafe { self.slice_unchecked(index.start, index.end) }
            } else {
                super::slice_error_fail(self, index.start, index.end)
            }
        }
    }

    /// Returns a mutable slice of the given string from the byte range
    /// [`begin`..`end`).
    #[stable(feature = "derefmut_for_string", since = "1.2.0")]
    impl ops::IndexMut<ops::Range<usize>> for str {
        #[inline]
        fn index_mut(&mut self, index: ops::Range<usize>) -> &mut str {
            // is_char_boundary checks that the index is in [0, .len()]
            if index.start <= index.end &&
               self.is_char_boundary(index.start) &&
               self.is_char_boundary(index.end) {
                unsafe { self.slice_mut_unchecked(index.start, index.end) }
            } else {
                super::slice_error_fail(self, index.start, index.end)
            }
        }
    }

    /// Returns a slice of the string from the beginning to byte
    /// `end`.
    ///
    /// Equivalent to `self[0 .. end]`.
    ///
    /// Panics when `end` does not point to a valid character, or is
    /// out of bounds.
    #[stable(feature = "rust1", since = "1.0.0")]
    impl ops::Index<ops::RangeTo<usize>> for str {
        type Output = str;

        #[inline]
        fn index(&self, index: ops::RangeTo<usize>) -> &str {
            // is_char_boundary checks that the index is in [0, .len()]
            if self.is_char_boundary(index.end) {
                unsafe { self.slice_unchecked(0, index.end) }
            } else {
                super::slice_error_fail(self, 0, index.end)
            }
        }
    }

    /// Returns a mutable slice of the string from the beginning to byte
    /// `end`.
    #[stable(feature = "derefmut_for_string", since = "1.2.0")]
    impl ops::IndexMut<ops::RangeTo<usize>> for str {
        #[inline]
        fn index_mut(&mut self, index: ops::RangeTo<usize>) -> &mut str {
            // is_char_boundary checks that the index is in [0, .len()]
            if self.is_char_boundary(index.end) {
                unsafe { self.slice_mut_unchecked(0, index.end) }
            } else {
                super::slice_error_fail(self, 0, index.end)
            }
        }
    }

    /// Returns a slice of the string from `begin` to its end.
    ///
    /// Equivalent to `self[begin .. self.len()]`.
    ///
    /// Panics when `begin` does not point to a valid character, or is
    /// out of bounds.
    #[stable(feature = "rust1", since = "1.0.0")]
    impl ops::Index<ops::RangeFrom<usize>> for str {
        type Output = str;

        #[inline]
        fn index(&self, index: ops::RangeFrom<usize>) -> &str {
            // is_char_boundary checks that the index is in [0, .len()]
            if self.is_char_boundary(index.start) {
                unsafe { self.slice_unchecked(index.start, self.len()) }
            } else {
                super::slice_error_fail(self, index.start, self.len())
            }
        }
    }

    /// Returns a slice of the string from `begin` to its end.
    #[stable(feature = "derefmut_for_string", since = "1.2.0")]
    impl ops::IndexMut<ops::RangeFrom<usize>> for str {
        #[inline]
        fn index_mut(&mut self, index: ops::RangeFrom<usize>) -> &mut str {
            // is_char_boundary checks that the index is in [0, .len()]
            if self.is_char_boundary(index.start) {
                let len = self.len();
                unsafe { self.slice_mut_unchecked(index.start, len) }
            } else {
                super::slice_error_fail(self, index.start, self.len())
            }
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl ops::Index<ops::RangeFull> for str {
        type Output = str;

        #[inline]
        fn index(&self, _index: ops::RangeFull) -> &str {
            self
        }
    }

    #[stable(feature = "derefmut_for_string", since = "1.2.0")]
    impl ops::IndexMut<ops::RangeFull> for str {
        #[inline]
        fn index_mut(&mut self, _index: ops::RangeFull) -> &mut str {
            self
        }
    }
}

/// Methods for string slices
#[allow(missing_docs)]
#[doc(hidden)]
#[unstable(feature = "core_str_ext",
           reason = "stable interface provided by `impl str` in later crates")]
pub trait StrExt {
    // NB there are no docs here are they're all located on the StrExt trait in
    // libcollections, not here.

    fn contains<'a, P: Pattern<'a>>(&'a self, pat: P) -> bool;
    fn contains_char<'a, P: Pattern<'a>>(&'a self, pat: P) -> bool;
    fn chars<'a>(&'a self) -> Chars<'a>;
    fn bytes<'a>(&'a self) -> Bytes<'a>;
    fn char_indices<'a>(&'a self) -> CharIndices<'a>;
    fn split<'a, P: Pattern<'a>>(&'a self, pat: P) -> Split<'a, P>;
    fn rsplit<'a, P: Pattern<'a>>(&'a self, pat: P) -> RSplit<'a, P>
        where P::Searcher: ReverseSearcher<'a>;
    fn splitn<'a, P: Pattern<'a>>(&'a self, count: usize, pat: P) -> SplitN<'a, P>;
    fn rsplitn<'a, P: Pattern<'a>>(&'a self, count: usize, pat: P) -> RSplitN<'a, P>
        where P::Searcher: ReverseSearcher<'a>;
    fn split_terminator<'a, P: Pattern<'a>>(&'a self, pat: P) -> SplitTerminator<'a, P>;
    fn rsplit_terminator<'a, P: Pattern<'a>>(&'a self, pat: P) -> RSplitTerminator<'a, P>
        where P::Searcher: ReverseSearcher<'a>;
    fn matches<'a, P: Pattern<'a>>(&'a self, pat: P) -> Matches<'a, P>;
    fn rmatches<'a, P: Pattern<'a>>(&'a self, pat: P) -> RMatches<'a, P>
        where P::Searcher: ReverseSearcher<'a>;
    fn match_indices<'a, P: Pattern<'a>>(&'a self, pat: P) -> MatchIndices<'a, P>;
    fn rmatch_indices<'a, P: Pattern<'a>>(&'a self, pat: P) -> RMatchIndices<'a, P>
        where P::Searcher: ReverseSearcher<'a>;
    fn lines<'a>(&'a self) -> Lines<'a>;
    fn lines_any<'a>(&'a self) -> LinesAny<'a>;
    fn char_len(&self) -> usize;
    fn slice_chars<'a>(&'a self, begin: usize, end: usize) -> &'a str;
    unsafe fn slice_unchecked<'a>(&'a self, begin: usize, end: usize) -> &'a str;
    unsafe fn slice_mut_unchecked<'a>(&'a mut self, begin: usize, end: usize) -> &'a mut str;
    fn starts_with<'a, P: Pattern<'a>>(&'a self, pat: P) -> bool;
    fn ends_with<'a, P: Pattern<'a>>(&'a self, pat: P) -> bool
        where P::Searcher: ReverseSearcher<'a>;
    fn trim_matches<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str
        where P::Searcher: DoubleEndedSearcher<'a>;
    fn trim_left_matches<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str;
    fn trim_right_matches<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str
        where P::Searcher: ReverseSearcher<'a>;
    fn is_char_boundary(&self, index: usize) -> bool;
    fn char_range_at(&self, start: usize) -> CharRange;
    fn char_range_at_reverse(&self, start: usize) -> CharRange;
    fn char_at(&self, i: usize) -> char;
    fn char_at_reverse(&self, i: usize) -> char;
    fn as_bytes<'a>(&'a self) -> &'a [u8];
    fn find<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<usize>;
    fn rfind<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<usize>
        where P::Searcher: ReverseSearcher<'a>;
    fn find_str<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<usize>;
    fn split_at(&self, mid: usize) -> (&str, &str);
    fn split_at_mut(&mut self, mid: usize) -> (&mut str, &mut str);
    fn slice_shift_char<'a>(&'a self) -> Option<(char, &'a str)>;
    fn subslice_offset(&self, inner: &str) -> usize;
    fn as_ptr(&self) -> *const u8;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn parse<T: FromStr>(&self) -> Result<T, T::Err>;
}

#[inline(never)]
fn slice_error_fail(s: &str, begin: usize, end: usize) -> ! {
    assert!(begin <= end);
    panic!("index {} and/or {} in `{}` do not lie on character boundary",
          begin, end, s);
}

impl StrExt for str {
    #[inline]
    fn contains<'a, P: Pattern<'a>>(&'a self, pat: P) -> bool {
        pat.is_contained_in(self)
    }

    #[inline]
    fn contains_char<'a, P: Pattern<'a>>(&'a self, pat: P) -> bool {
        pat.is_contained_in(self)
    }

    #[inline]
    fn chars(&self) -> Chars {
        Chars{iter: self.as_bytes().iter()}
    }

    #[inline]
    fn bytes(&self) -> Bytes {
        Bytes(self.as_bytes().iter().map(BytesDeref))
    }

    #[inline]
    fn char_indices(&self) -> CharIndices {
        CharIndices { front_offset: 0, iter: self.chars() }
    }

    #[inline]
    fn split<'a, P: Pattern<'a>>(&'a self, pat: P) -> Split<'a, P> {
        Split(SplitInternal {
            start: 0,
            end: self.len(),
            matcher: pat.into_searcher(self),
            allow_trailing_empty: true,
            finished: false,
        })
    }

    #[inline]
    fn rsplit<'a, P: Pattern<'a>>(&'a self, pat: P) -> RSplit<'a, P>
        where P::Searcher: ReverseSearcher<'a>
    {
        RSplit(self.split(pat).0)
    }

    #[inline]
    fn splitn<'a, P: Pattern<'a>>(&'a self, count: usize, pat: P) -> SplitN<'a, P> {
        SplitN(SplitNInternal {
            iter: self.split(pat).0,
            count: count,
        })
    }

    #[inline]
    fn rsplitn<'a, P: Pattern<'a>>(&'a self, count: usize, pat: P) -> RSplitN<'a, P>
        where P::Searcher: ReverseSearcher<'a>
    {
        RSplitN(self.splitn(count, pat).0)
    }

    #[inline]
    fn split_terminator<'a, P: Pattern<'a>>(&'a self, pat: P) -> SplitTerminator<'a, P> {
        SplitTerminator(SplitInternal {
            allow_trailing_empty: false,
            ..self.split(pat).0
        })
    }

    #[inline]
    fn rsplit_terminator<'a, P: Pattern<'a>>(&'a self, pat: P) -> RSplitTerminator<'a, P>
        where P::Searcher: ReverseSearcher<'a>
    {
        RSplitTerminator(self.split_terminator(pat).0)
    }

    #[inline]
    fn matches<'a, P: Pattern<'a>>(&'a self, pat: P) -> Matches<'a, P> {
        Matches(MatchesInternal(pat.into_searcher(self)))
    }

    #[inline]
    fn rmatches<'a, P: Pattern<'a>>(&'a self, pat: P) -> RMatches<'a, P>
        where P::Searcher: ReverseSearcher<'a>
    {
        RMatches(self.matches(pat).0)
    }

    #[inline]
    fn match_indices<'a, P: Pattern<'a>>(&'a self, pat: P) -> MatchIndices<'a, P> {
        MatchIndices(MatchIndicesInternal(pat.into_searcher(self)))
    }

    #[inline]
    fn rmatch_indices<'a, P: Pattern<'a>>(&'a self, pat: P) -> RMatchIndices<'a, P>
        where P::Searcher: ReverseSearcher<'a>
    {
        RMatchIndices(self.match_indices(pat).0)
    }
    #[inline]
    fn lines(&self) -> Lines {
        Lines(self.split_terminator('\n'))
    }

    #[inline]
    fn lines_any(&self) -> LinesAny {
        LinesAny(self.lines().map(LinesAnyMap))
    }

    #[inline]
    fn char_len(&self) -> usize { self.chars().count() }

    fn slice_chars(&self, begin: usize, end: usize) -> &str {
        assert!(begin <= end);
        let mut count = 0;
        let mut begin_byte = None;
        let mut end_byte = None;

        // This could be even more efficient by not decoding,
        // only finding the char boundaries
        for (idx, _) in self.char_indices() {
            if count == begin { begin_byte = Some(idx); }
            if count == end { end_byte = Some(idx); break; }
            count += 1;
        }
        if begin_byte.is_none() && count == begin { begin_byte = Some(self.len()) }
        if end_byte.is_none() && count == end { end_byte = Some(self.len()) }

        match (begin_byte, end_byte) {
            (None, _) => panic!("slice_chars: `begin` is beyond end of string"),
            (_, None) => panic!("slice_chars: `end` is beyond end of string"),
            (Some(a), Some(b)) => unsafe { self.slice_unchecked(a, b) }
        }
    }

    #[inline]
    unsafe fn slice_unchecked(&self, begin: usize, end: usize) -> &str {
        mem::transmute(Slice {
            data: self.as_ptr().offset(begin as isize),
            len: end - begin,
        })
    }

    #[inline]
    unsafe fn slice_mut_unchecked(&mut self, begin: usize, end: usize) -> &mut str {
        mem::transmute(Slice {
            data: self.as_ptr().offset(begin as isize),
            len: end - begin,
        })
    }

    #[inline]
    fn starts_with<'a, P: Pattern<'a>>(&'a self, pat: P) -> bool {
        pat.is_prefix_of(self)
    }

    #[inline]
    fn ends_with<'a, P: Pattern<'a>>(&'a self, pat: P) -> bool
        where P::Searcher: ReverseSearcher<'a>
    {
        pat.is_suffix_of(self)
    }

    #[inline]
    fn trim_matches<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str
        where P::Searcher: DoubleEndedSearcher<'a>
    {
        let mut i = 0;
        let mut j = 0;
        let mut matcher = pat.into_searcher(self);
        if let Some((a, b)) = matcher.next_reject() {
            i = a;
            j = b; // Rember earliest known match, correct it below if
                   // last match is different
        }
        if let Some((_, b)) = matcher.next_reject_back() {
            j = b;
        }
        unsafe {
            // Searcher is known to return valid indices
            self.slice_unchecked(i, j)
        }
    }

    #[inline]
    fn trim_left_matches<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str {
        let mut i = self.len();
        let mut matcher = pat.into_searcher(self);
        if let Some((a, _)) = matcher.next_reject() {
            i = a;
        }
        unsafe {
            // Searcher is known to return valid indices
            self.slice_unchecked(i, self.len())
        }
    }

    #[inline]
    fn trim_right_matches<'a, P: Pattern<'a>>(&'a self, pat: P) -> &'a str
        where P::Searcher: ReverseSearcher<'a>
    {
        let mut j = 0;
        let mut matcher = pat.into_searcher(self);
        if let Some((_, b)) = matcher.next_reject_back() {
            j = b;
        }
        unsafe {
            // Searcher is known to return valid indices
            self.slice_unchecked(0, j)
        }
    }

    #[inline]
    fn is_char_boundary(&self, index: usize) -> bool {
        if index == self.len() { return true; }
        match self.as_bytes().get(index) {
            None => false,
            Some(&b) => b < 128 || b >= 192,
        }
    }

    #[inline]
    fn char_range_at(&self, i: usize) -> CharRange {
        let (c, n) = char_range_at_raw(self.as_bytes(), i);
        CharRange { ch: unsafe { mem::transmute(c) }, next: n }
    }

    #[inline]
    fn char_range_at_reverse(&self, start: usize) -> CharRange {
        let mut prev = start;

        prev = prev.saturating_sub(1);
        if self.as_bytes()[prev] < 128 {
            return CharRange{ch: self.as_bytes()[prev] as char, next: prev}
        }

        // Multibyte case is a fn to allow char_range_at_reverse to inline cleanly
        fn multibyte_char_range_at_reverse(s: &str, mut i: usize) -> CharRange {
            // while there is a previous byte == 10......
            while i > 0 && s.as_bytes()[i] & !CONT_MASK == TAG_CONT_U8 {
                i -= 1;
            }

            let first= s.as_bytes()[i];
            let w = UTF8_CHAR_WIDTH[first as usize];
            assert!(w != 0);

            let mut val = utf8_first_byte(first, w as u32);
            val = utf8_acc_cont_byte(val, s.as_bytes()[i + 1]);
            if w > 2 { val = utf8_acc_cont_byte(val, s.as_bytes()[i + 2]); }
            if w > 3 { val = utf8_acc_cont_byte(val, s.as_bytes()[i + 3]); }

            return CharRange {ch: unsafe { mem::transmute(val) }, next: i};
        }

        return multibyte_char_range_at_reverse(self, prev);
    }

    #[inline]
    fn char_at(&self, i: usize) -> char {
        self.char_range_at(i).ch
    }

    #[inline]
    fn char_at_reverse(&self, i: usize) -> char {
        self.char_range_at_reverse(i).ch
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe { mem::transmute(self) }
    }

    fn find<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<usize> {
        pat.into_searcher(self).next_match().map(|(i, _)| i)
    }

    fn rfind<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<usize>
        where P::Searcher: ReverseSearcher<'a>
    {
        pat.into_searcher(self).next_match_back().map(|(i, _)| i)
    }

    fn find_str<'a, P: Pattern<'a>>(&'a self, pat: P) -> Option<usize> {
        self.find(pat)
    }

    fn split_at(&self, mid: usize) -> (&str, &str) {
        // is_char_boundary checks that the index is in [0, .len()]
        if self.is_char_boundary(mid) {
            unsafe {
                (self.slice_unchecked(0, mid),
                 self.slice_unchecked(mid, self.len()))
            }
        } else {
            slice_error_fail(self, 0, mid)
        }
    }

    fn split_at_mut(&mut self, mid: usize) -> (&mut str, &mut str) {
        // is_char_boundary checks that the index is in [0, .len()]
        if self.is_char_boundary(mid) {
            let len = self.len();
            unsafe {
                let self2: &mut str = mem::transmute_copy(&self);
                (self.slice_mut_unchecked(0, mid),
                 self2.slice_mut_unchecked(mid, len))
            }
        } else {
            slice_error_fail(self, 0, mid)
        }
    }

    #[inline]
    fn slice_shift_char(&self) -> Option<(char, &str)> {
        if self.is_empty() {
            None
        } else {
            let ch = self.char_at(0);
            let next_s = unsafe { self.slice_unchecked(ch.len_utf8(), self.len()) };
            Some((ch, next_s))
        }
    }

    fn subslice_offset(&self, inner: &str) -> usize {
        let a_start = self.as_ptr() as usize;
        let a_end = a_start + self.len();
        let b_start = inner.as_ptr() as usize;
        let b_end = b_start + inner.len();

        assert!(a_start <= b_start);
        assert!(b_end <= a_end);
        b_start - a_start
    }

    #[inline]
    fn as_ptr(&self) -> *const u8 {
        self.repr().data
    }

    #[inline]
    fn len(&self) -> usize { self.repr().len }

    #[inline]
    fn is_empty(&self) -> bool { self.len() == 0 }

    #[inline]
    fn parse<T: FromStr>(&self) -> Result<T, T::Err> { FromStr::from_str(self) }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl AsRef<[u8]> for str {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Pluck a code point out of a UTF-8-like byte slice and return the
/// index of the next code point.
#[inline]
fn char_range_at_raw(bytes: &[u8], i: usize) -> (u32, usize) {
    if bytes[i] < 128 {
        return (bytes[i] as u32, i + 1);
    }

    // Multibyte case is a fn to allow char_range_at to inline cleanly
    fn multibyte_char_range_at(bytes: &[u8], i: usize) -> (u32, usize) {
        let first = bytes[i];
        let w = UTF8_CHAR_WIDTH[first as usize];
        assert!(w != 0);

        let mut val = utf8_first_byte(first, w as u32);
        val = utf8_acc_cont_byte(val, bytes[i + 1]);
        if w > 2 { val = utf8_acc_cont_byte(val, bytes[i + 2]); }
        if w > 3 { val = utf8_acc_cont_byte(val, bytes[i + 3]); }

        return (val, i + w as usize);
    }

    multibyte_char_range_at(bytes, i)
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Default for &'a str {
    #[stable(feature = "rust1", since = "1.0.0")]
    fn default() -> &'a str { "" }
}
