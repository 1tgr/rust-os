// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Integer and floating-point number formatting

// FIXME: #6220 Implement floating point formatting

#![allow(unsigned_negation)]

use prelude::*;

use fmt;
use num::Zero;
use ops::{Div, Rem, Sub};
use str;

#[doc(hidden)]
trait Int: Zero + PartialEq + PartialOrd + Div<Output=Self> + Rem<Output=Self> +
           Sub<Output=Self> + Copy {
    fn from_u8(u: u8) -> Self;
    fn to_u8(&self) -> u8;
}

macro_rules! doit {
    ($($t:ident)*) => ($(impl Int for $t {
        fn from_u8(u: u8) -> $t { u as $t }
        fn to_u8(&self) -> u8 { *self as u8 }
    })*)
}
doit! { i8 i16 i32 i64 isize u8 u16 u32 u64 usize }

/// A type that represents a specific radix
#[doc(hidden)]
trait GenericRadix {
    /// The number of digits.
    fn base(&self) -> u8;

    /// A radix-specific prefix string.
    fn prefix(&self) -> &'static str { "" }

    /// Converts an integer to corresponding radix digit.
    fn digit(&self, x: u8) -> u8;

    /// Format an integer using the radix using a formatter.
    fn fmt_int<T: Int>(&self, mut x: T, f: &mut fmt::Formatter) -> fmt::Result {
        // The radix can be as low as 2, so we need a buffer of at least 64
        // characters for a base 2 number.
        let zero = T::zero();
        let is_positive = x >= zero;
        let mut buf = [0; 64];
        let mut curr = buf.len();
        let base = T::from_u8(self.base());
        if is_positive {
            // Accumulate each digit of the number from the least significant
            // to the most significant figure.
            for byte in buf.iter_mut().rev() {
                let n = x % base;              // Get the current place value.
                x = x / base;                  // Deaccumulate the number.
                *byte = self.digit(n.to_u8()); // Store the digit in the buffer.
                curr -= 1;
                if x == zero { break };        // No more digits left to accumulate.
            }
        } else {
            // Do the same as above, but accounting for two's complement.
            for byte in buf.iter_mut().rev() {
                let n = zero - (x % base);     // Get the current place value.
                x = x / base;                  // Deaccumulate the number.
                *byte = self.digit(n.to_u8()); // Store the digit in the buffer.
                curr -= 1;
                if x == zero { break };        // No more digits left to accumulate.
            }
        }
        let buf = unsafe { str::from_utf8_unchecked(&buf[curr..]) };
        f.pad_integral(is_positive, self.prefix(), buf)
    }
}

/// A binary (base 2) radix
#[derive(Clone, PartialEq)]
struct Binary;

/// An octal (base 8) radix
#[derive(Clone, PartialEq)]
struct Octal;

/// A decimal (base 10) radix
#[derive(Clone, PartialEq)]
struct Decimal;

/// A hexadecimal (base 16) radix, formatted with lower-case characters
#[derive(Clone, PartialEq)]
struct LowerHex;

/// A hexadecimal (base 16) radix, formatted with upper-case characters
#[derive(Clone, PartialEq)]
struct UpperHex;

macro_rules! radix {
    ($T:ident, $base:expr, $prefix:expr, $($x:pat => $conv:expr),+) => {
        impl GenericRadix for $T {
            fn base(&self) -> u8 { $base }
            fn prefix(&self) -> &'static str { $prefix }
            fn digit(&self, x: u8) -> u8 {
                match x {
                    $($x => $conv,)+
                    x => panic!("number not in the range 0..{}: {}", self.base() - 1, x),
                }
            }
        }
    }
}

radix! { Binary,    2, "0b", x @  0 ...  2 => b'0' + x }
radix! { Octal,     8, "0o", x @  0 ...  7 => b'0' + x }
radix! { Decimal,  10, "",   x @  0 ...  9 => b'0' + x }
radix! { LowerHex, 16, "0x", x @  0 ...  9 => b'0' + x,
                             x @ 10 ... 15 => b'a' + (x - 10) }
radix! { UpperHex, 16, "0x", x @  0 ...  9 => b'0' + x,
                             x @ 10 ... 15 => b'A' + (x - 10) }

/// A radix with in the range of `2..36`.
#[derive(Clone, Copy, PartialEq)]
#[unstable(feature = "core",
           reason = "may be renamed or move to a different module")]
pub struct Radix {
    base: u8,
}

impl Radix {
    fn new(base: u8) -> Radix {
        assert!(2 <= base && base <= 36, "the base must be in the range of 2..36: {}", base);
        Radix { base: base }
    }
}

impl GenericRadix for Radix {
    fn base(&self) -> u8 { self.base }
    fn digit(&self, x: u8) -> u8 {
        match x {
            x @  0 ... 9 => b'0' + x,
            x if x < self.base() => b'a' + (x - 10),
            x => panic!("number not in the range 0..{}: {}", self.base() - 1, x),
        }
    }
}

/// A helper type for formatting radixes.
#[unstable(feature = "core",
           reason = "may be renamed or move to a different module")]
#[derive(Copy, Clone)]
pub struct RadixFmt<T, R>(T, R);

/// Constructs a radix formatter in the range of `2..36`.
///
/// # Examples
///
/// ```
/// # #![feature(core)]
/// use std::fmt::radix;
/// assert_eq!(format!("{}", radix(55, 36)), "1j".to_string());
/// ```
#[unstable(feature = "core",
           reason = "may be renamed or move to a different module")]
pub fn radix<T>(x: T, base: u8) -> RadixFmt<T, Radix> {
    RadixFmt(x, Radix::new(base))
}

macro_rules! radix_fmt {
    ($T:ty as $U:ty, $fmt:ident) => {
        #[stable(feature = "rust1", since = "1.0.0")]
        impl fmt::Debug for RadixFmt<$T, Radix> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(self, f)
            }
        }
        #[stable(feature = "rust1", since = "1.0.0")]
        impl fmt::Display for RadixFmt<$T, Radix> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self { RadixFmt(ref x, radix) => radix.$fmt(*x as $U, f) }
            }
        }
    }
}
macro_rules! int_base {
    ($Trait:ident for $T:ident as $U:ident -> $Radix:ident) => {
        #[stable(feature = "rust1", since = "1.0.0")]
        impl fmt::$Trait for $T {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                $Radix.fmt_int(*self as $U, f)
            }
        }
    }
}

macro_rules! debug {
    ($T:ident) => {
        #[stable(feature = "rust1", since = "1.0.0")]
        impl fmt::Debug for $T {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(self, f)
            }
        }
    }
}
macro_rules! integer {
    ($Int:ident, $Uint:ident) => {
        int_base! { Display  for $Int as $Int   -> Decimal }
        int_base! { Binary   for $Int as $Uint  -> Binary }
        int_base! { Octal    for $Int as $Uint  -> Octal }
        int_base! { LowerHex for $Int as $Uint  -> LowerHex }
        int_base! { UpperHex for $Int as $Uint  -> UpperHex }
        radix_fmt! { $Int as $Int, fmt_int }
        debug! { $Int }

        int_base! { Display  for $Uint as $Uint -> Decimal }
        int_base! { Binary   for $Uint as $Uint -> Binary }
        int_base! { Octal    for $Uint as $Uint -> Octal }
        int_base! { LowerHex for $Uint as $Uint -> LowerHex }
        int_base! { UpperHex for $Uint as $Uint -> UpperHex }
        radix_fmt! { $Uint as $Uint, fmt_int }
        debug! { $Uint }
    }
}
integer! { isize, usize }
integer! { i8, u8 }
integer! { i16, u16 }
integer! { i32, u32 }
integer! { i64, u64 }
