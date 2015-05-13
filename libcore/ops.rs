// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Overloadable operators
//!
//! Implementing these traits allows you to get an effect similar to
//! overloading operators.
//!
//! Some of these traits are imported by the prelude, so they are available in
//! every Rust program.
//!
//! Many of the operators take their operands by value. In non-generic
//! contexts involving built-in types, this is usually not a problem.
//! However, using these operators in generic code, requires some
//! attention if values have to be reused as opposed to letting the operators
//! consume them. One option is to occasionally use `clone()`.
//! Another option is to rely on the types involved providing additional
//! operator implementations for references. For example, for a user-defined
//! type `T` which is supposed to support addition, it is probably a good
//! idea to have both `T` and `&T` implement the traits `Add<T>` and `Add<&T>`
//! so that generic code can be written without unnecessary cloning.
//!
//! # Examples
//!
//! This example creates a `Point` struct that implements `Add` and `Sub`, and then
//! demonstrates adding and subtracting two `Point`s.
//!
//! ```rust
//! use std::ops::{Add, Sub};
//!
//! #[derive(Debug)]
//! struct Point {
//!     x: i32,
//!     y: i32
//! }
//!
//! impl Add for Point {
//!     type Output = Point;
//!
//!     fn add(self, other: Point) -> Point {
//!         Point {x: self.x + other.x, y: self.y + other.y}
//!     }
//! }
//!
//! impl Sub for Point {
//!     type Output = Point;
//!
//!     fn sub(self, other: Point) -> Point {
//!         Point {x: self.x - other.x, y: self.y - other.y}
//!     }
//! }
//! fn main() {
//!     println!("{:?}", Point {x: 1, y: 0} + Point {x: 2, y: 3});
//!     println!("{:?}", Point {x: 1, y: 0} - Point {x: 2, y: 3});
//! }
//! ```
//!
//! See the documentation for each trait for a minimum implementation that prints
//! something to the screen.

#![stable(feature = "rust1", since = "1.0.0")]

use marker::Sized;
use fmt;

/// The `Drop` trait is used to run some code when a value goes out of scope. This
/// is sometimes called a 'destructor'.
///
/// # Examples
///
/// A trivial implementation of `Drop`. The `drop` method is called when `_x` goes
/// out of scope, and therefore `main` prints `Dropping!`.
///
/// ```
/// struct HasDrop;
///
/// impl Drop for HasDrop {
///     fn drop(&mut self) {
///         println!("Dropping!");
///     }
/// }
///
/// fn main() {
///     let _x = HasDrop;
/// }
/// ```
#[lang="drop"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Drop {
    /// The `drop` method, called when the value goes out of scope.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn drop(&mut self);
}

// implements the unary operator "op &T"
// based on "op T" where T is expected to be `Copy`able
macro_rules! forward_ref_unop {
    (impl $imp:ident, $method:ident for $t:ty) => {
        #[unstable(feature = "core",
                   reason = "recently added, waiting for dust to settle")]
        impl<'a> $imp for &'a $t {
            type Output = <$t as $imp>::Output;

            #[inline]
            fn $method(self) -> <$t as $imp>::Output {
                $imp::$method(*self)
            }
        }
    }
}

// implements binary operators "&T op U", "T op &U", "&T op &U"
// based on "T op U" where T and U are expected to be `Copy`able
macro_rules! forward_ref_binop {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        #[unstable(feature = "core",
                   reason = "recently added, waiting for dust to settle")]
        impl<'a> $imp<$u> for &'a $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, other)
            }
        }

        #[unstable(feature = "core",
                   reason = "recently added, waiting for dust to settle")]
        impl<'a> $imp<&'a $u> for $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &'a $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(self, *other)
            }
        }

        #[unstable(feature = "core",
                   reason = "recently added, waiting for dust to settle")]
        impl<'a, 'b> $imp<&'a $u> for &'b $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &'a $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, *other)
            }
        }
    }
}

/// The `Add` trait is used to specify the functionality of `+`.
///
/// # Examples
///
/// A trivial implementation of `Add`. When `Foo + Foo` happens, it ends up
/// calling `add`, and therefore, `main` prints `Adding!`.
///
/// ```
/// use std::ops::Add;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl Add for Foo {
///     type Output = Foo;
///
///     fn add(self, _rhs: Foo) -> Foo {
///         println!("Adding!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo + Foo;
/// }
/// ```
#[lang="add"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Add<RHS=Self> {
    /// The resulting type after applying the `+` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the `+` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn add(self, rhs: RHS) -> Self::Output;
}

macro_rules! add_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl Add for $t {
            type Output = $t;

            #[inline]
            fn add(self, other: $t) -> $t { self + other }
        }

        forward_ref_binop! { impl Add, add for $t, $t }
    )*)
}

add_impl! { usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 }

/// The `Sub` trait is used to specify the functionality of `-`.
///
/// # Examples
///
/// A trivial implementation of `Sub`. When `Foo - Foo` happens, it ends up
/// calling `sub`, and therefore, `main` prints `Subtracting!`.
///
/// ```
/// use std::ops::Sub;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl Sub for Foo {
///     type Output = Foo;
///
///     fn sub(self, _rhs: Foo) -> Foo {
///         println!("Subtracting!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo - Foo;
/// }
/// ```
#[lang="sub"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Sub<RHS=Self> {
    /// The resulting type after applying the `-` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the `-` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn sub(self, rhs: RHS) -> Self::Output;
}

macro_rules! sub_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl Sub for $t {
            type Output = $t;

            #[inline]
            fn sub(self, other: $t) -> $t { self - other }
        }

        forward_ref_binop! { impl Sub, sub for $t, $t }
    )*)
}

sub_impl! { usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 }

/// The `Mul` trait is used to specify the functionality of `*`.
///
/// # Examples
///
/// A trivial implementation of `Mul`. When `Foo * Foo` happens, it ends up
/// calling `mul`, and therefore, `main` prints `Multiplying!`.
///
/// ```
/// use std::ops::Mul;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl Mul for Foo {
///     type Output = Foo;
///
///     fn mul(self, _rhs: Foo) -> Foo {
///         println!("Multiplying!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo * Foo;
/// }
/// ```
#[lang="mul"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Mul<RHS=Self> {
    /// The resulting type after applying the `*` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the `*` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn mul(self, rhs: RHS) -> Self::Output;
}

macro_rules! mul_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl Mul for $t {
            type Output = $t;

            #[inline]
            fn mul(self, other: $t) -> $t { self * other }
        }

        forward_ref_binop! { impl Mul, mul for $t, $t }
    )*)
}

mul_impl! { usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 }

/// The `Div` trait is used to specify the functionality of `/`.
///
/// # Examples
///
/// A trivial implementation of `Div`. When `Foo / Foo` happens, it ends up
/// calling `div`, and therefore, `main` prints `Dividing!`.
///
/// ```
/// use std::ops::Div;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl Div for Foo {
///     type Output = Foo;
///
///     fn div(self, _rhs: Foo) -> Foo {
///         println!("Dividing!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo / Foo;
/// }
/// ```
#[lang="div"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Div<RHS=Self> {
    /// The resulting type after applying the `/` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the `/` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn div(self, rhs: RHS) -> Self::Output;
}

macro_rules! div_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl Div for $t {
            type Output = $t;

            #[inline]
            fn div(self, other: $t) -> $t { self / other }
        }

        forward_ref_binop! { impl Div, div for $t, $t }
    )*)
}

div_impl! { usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 }

/// The `Rem` trait is used to specify the functionality of `%`.
///
/// # Examples
///
/// A trivial implementation of `Rem`. When `Foo % Foo` happens, it ends up
/// calling `rem`, and therefore, `main` prints `Remainder-ing!`.
///
/// ```
/// use std::ops::Rem;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl Rem for Foo {
///     type Output = Foo;
///
///     fn rem(self, _rhs: Foo) -> Foo {
///         println!("Remainder-ing!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo % Foo;
/// }
/// ```
#[lang="rem"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Rem<RHS=Self> {
    /// The resulting type after applying the `%` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output = Self;

    /// The method for the `%` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn rem(self, rhs: RHS) -> Self::Output;
}

macro_rules! rem_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl Rem for $t {
            type Output = $t;

            #[inline]
            fn rem(self, other: $t) -> $t { self % other }
        }

        forward_ref_binop! { impl Rem, rem for $t, $t }
    )*)
}

macro_rules! rem_float_impl {
    ($t:ty, $fmod:ident) => {
        #[stable(feature = "rust1", since = "1.0.0")]
        impl Rem for $t {
            type Output = $t;

            #[inline]
            fn rem(self, other: $t) -> $t {
                extern { fn $fmod(a: $t, b: $t) -> $t; }
                unsafe { $fmod(self, other) }
            }
        }

        forward_ref_binop! { impl Rem, rem for $t, $t }
    }
}

rem_impl! { usize u8 u16 u32 u64 isize i8 i16 i32 i64 }
rem_float_impl! { f32, fmodf }
rem_float_impl! { f64, fmod }

/// The `Neg` trait is used to specify the functionality of unary `-`.
///
/// # Examples
///
/// A trivial implementation of `Neg`. When `-Foo` happens, it ends up calling
/// `neg`, and therefore, `main` prints `Negating!`.
///
/// ```
/// use std::ops::Neg;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl Neg for Foo {
///     type Output = Foo;
///
///     fn neg(self) -> Foo {
///         println!("Negating!");
///         self
///     }
/// }
///
/// fn main() {
///     -Foo;
/// }
/// ```
#[lang="neg"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Neg {
    /// The resulting type after applying the `-` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the unary `-` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn neg(self) -> Self::Output;
}



macro_rules! neg_impl_core {
    ($id:ident => $body:expr, $($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        #[allow(unsigned_negation)]
        impl Neg for $t {
            #[stable(feature = "rust1", since = "1.0.0")]
            type Output = $t;

            #[inline]
            #[stable(feature = "rust1", since = "1.0.0")]
            fn neg(self) -> $t { let $id = self; $body }
        }

        forward_ref_unop! { impl Neg, neg for $t }
    )*)
}

macro_rules! neg_impl_numeric {
    ($($t:ty)*) => { neg_impl_core!{ x => -x, $($t)*} }
}

macro_rules! neg_impl_unsigned {
    ($($t:ty)*) => {
        neg_impl_core!{ x => {
            !x.wrapping_add(1)
        }, $($t)*} }
}

// neg_impl_unsigned! { usize u8 u16 u32 u64 }
neg_impl_numeric! { isize i8 i16 i32 i64 f32 f64 }

/// The `Not` trait is used to specify the functionality of unary `!`.
///
/// # Examples
///
/// A trivial implementation of `Not`. When `!Foo` happens, it ends up calling
/// `not`, and therefore, `main` prints `Not-ing!`.
///
/// ```
/// use std::ops::Not;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl Not for Foo {
///     type Output = Foo;
///
///     fn not(self) -> Foo {
///         println!("Not-ing!");
///         self
///     }
/// }
///
/// fn main() {
///     !Foo;
/// }
/// ```
#[lang="not"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Not {
    /// The resulting type after applying the `!` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the unary `!` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn not(self) -> Self::Output;
}

macro_rules! not_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl Not for $t {
            type Output = $t;

            #[inline]
            fn not(self) -> $t { !self }
        }

        forward_ref_unop! { impl Not, not for $t }
    )*)
}

not_impl! { bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 }

/// The `BitAnd` trait is used to specify the functionality of `&`.
///
/// # Examples
///
/// A trivial implementation of `BitAnd`. When `Foo & Foo` happens, it ends up
/// calling `bitand`, and therefore, `main` prints `Bitwise And-ing!`.
///
/// ```
/// use std::ops::BitAnd;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl BitAnd for Foo {
///     type Output = Foo;
///
///     fn bitand(self, _rhs: Foo) -> Foo {
///         println!("Bitwise And-ing!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo & Foo;
/// }
/// ```
#[lang="bitand"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait BitAnd<RHS=Self> {
    /// The resulting type after applying the `&` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the `&` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn bitand(self, rhs: RHS) -> Self::Output;
}

macro_rules! bitand_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl BitAnd for $t {
            type Output = $t;

            #[inline]
            fn bitand(self, rhs: $t) -> $t { self & rhs }
        }

        forward_ref_binop! { impl BitAnd, bitand for $t, $t }
    )*)
}

bitand_impl! { bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 }

/// The `BitOr` trait is used to specify the functionality of `|`.
///
/// # Examples
///
/// A trivial implementation of `BitOr`. When `Foo | Foo` happens, it ends up
/// calling `bitor`, and therefore, `main` prints `Bitwise Or-ing!`.
///
/// ```
/// use std::ops::BitOr;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl BitOr for Foo {
///     type Output = Foo;
///
///     fn bitor(self, _rhs: Foo) -> Foo {
///         println!("Bitwise Or-ing!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo | Foo;
/// }
/// ```
#[lang="bitor"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait BitOr<RHS=Self> {
    /// The resulting type after applying the `|` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the `|` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn bitor(self, rhs: RHS) -> Self::Output;
}

macro_rules! bitor_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl BitOr for $t {
            type Output = $t;

            #[inline]
            fn bitor(self, rhs: $t) -> $t { self | rhs }
        }

        forward_ref_binop! { impl BitOr, bitor for $t, $t }
    )*)
}

bitor_impl! { bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 }

/// The `BitXor` trait is used to specify the functionality of `^`.
///
/// # Examples
///
/// A trivial implementation of `BitXor`. When `Foo ^ Foo` happens, it ends up
/// calling `bitxor`, and therefore, `main` prints `Bitwise Xor-ing!`.
///
/// ```
/// use std::ops::BitXor;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl BitXor for Foo {
///     type Output = Foo;
///
///     fn bitxor(self, _rhs: Foo) -> Foo {
///         println!("Bitwise Xor-ing!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo ^ Foo;
/// }
/// ```
#[lang="bitxor"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait BitXor<RHS=Self> {
    /// The resulting type after applying the `^` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the `^` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn bitxor(self, rhs: RHS) -> Self::Output;
}

macro_rules! bitxor_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl BitXor for $t {
            type Output = $t;

            #[inline]
            fn bitxor(self, other: $t) -> $t { self ^ other }
        }

        forward_ref_binop! { impl BitXor, bitxor for $t, $t }
    )*)
}

bitxor_impl! { bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 }

/// The `Shl` trait is used to specify the functionality of `<<`.
///
/// # Examples
///
/// A trivial implementation of `Shl`. When `Foo << Foo` happens, it ends up
/// calling `shl`, and therefore, `main` prints `Shifting left!`.
///
/// ```
/// use std::ops::Shl;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl Shl<Foo> for Foo {
///     type Output = Foo;
///
///     fn shl(self, _rhs: Foo) -> Foo {
///         println!("Shifting left!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo << Foo;
/// }
/// ```
#[lang="shl"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Shl<RHS> {
    /// The resulting type after applying the `<<` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the `<<` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn shl(self, rhs: RHS) -> Self::Output;
}

macro_rules! shl_impl {
    ($t:ty, $f:ty) => (
        #[stable(feature = "rust1", since = "1.0.0")]
        impl Shl<$f> for $t {
            type Output = $t;

            #[inline]
            fn shl(self, other: $f) -> $t {
                self << other
            }
        }

        forward_ref_binop! { impl Shl, shl for $t, $f }
    )
}

macro_rules! shl_impl_all {
    ($($t:ty)*) => ($(
        shl_impl! { $t, u8 }
        shl_impl! { $t, u16 }
        shl_impl! { $t, u32 }
        shl_impl! { $t, u64 }
        shl_impl! { $t, usize }

        shl_impl! { $t, i8 }
        shl_impl! { $t, i16 }
        shl_impl! { $t, i32 }
        shl_impl! { $t, i64 }
        shl_impl! { $t, isize }
    )*)
}

shl_impl_all! { u8 u16 u32 u64 usize i8 i16 i32 i64 isize }

/// The `Shr` trait is used to specify the functionality of `>>`.
///
/// # Examples
///
/// A trivial implementation of `Shr`. When `Foo >> Foo` happens, it ends up
/// calling `shr`, and therefore, `main` prints `Shifting right!`.
///
/// ```
/// use std::ops::Shr;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
///
/// impl Shr<Foo> for Foo {
///     type Output = Foo;
///
///     fn shr(self, _rhs: Foo) -> Foo {
///         println!("Shifting right!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo >> Foo;
/// }
/// ```
#[lang="shr"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Shr<RHS> {
    /// The resulting type after applying the `>>` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// The method for the `>>` operator
    #[stable(feature = "rust1", since = "1.0.0")]
    fn shr(self, rhs: RHS) -> Self::Output;
}

macro_rules! shr_impl {
    ($t:ty, $f:ty) => (
        impl Shr<$f> for $t {
            type Output = $t;

            #[inline]
            fn shr(self, other: $f) -> $t {
                self >> other
            }
        }

        forward_ref_binop! { impl Shr, shr for $t, $f }
    )
}

macro_rules! shr_impl_all {
    ($($t:ty)*) => ($(
        shr_impl! { $t, u8 }
        shr_impl! { $t, u16 }
        shr_impl! { $t, u32 }
        shr_impl! { $t, u64 }
        shr_impl! { $t, usize }

        shr_impl! { $t, i8 }
        shr_impl! { $t, i16 }
        shr_impl! { $t, i32 }
        shr_impl! { $t, i64 }
        shr_impl! { $t, isize }
    )*)
}

shr_impl_all! { u8 u16 u32 u64 usize i8 i16 i32 i64 isize }

/// The `Index` trait is used to specify the functionality of indexing operations
/// like `arr[idx]` when used in an immutable context.
///
/// # Examples
///
/// A trivial implementation of `Index`. When `Foo[Bar]` happens, it ends up
/// calling `index`, and therefore, `main` prints `Indexing!`.
///
/// ```
/// use std::ops::Index;
///
/// #[derive(Copy, Clone)]
/// struct Foo;
/// struct Bar;
///
/// impl Index<Bar> for Foo {
///     type Output = Foo;
///
///     fn index<'a>(&'a self, _index: Bar) -> &'a Foo {
///         println!("Indexing!");
///         self
///     }
/// }
///
/// fn main() {
///     Foo[Bar];
/// }
/// ```
#[lang="index"]
#[rustc_on_unimplemented = "the type `{Self}` cannot be indexed by `{Idx}`"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Index<Idx: ?Sized> {
    /// The returned type after indexing
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output: ?Sized;

    /// The method for the indexing (`Foo[Bar]`) operation
    #[stable(feature = "rust1", since = "1.0.0")]
    fn index<'a>(&'a self, index: Idx) -> &'a Self::Output;
}

/// The `IndexMut` trait is used to specify the functionality of indexing
/// operations like `arr[idx]`, when used in a mutable context.
///
/// # Examples
///
/// A trivial implementation of `IndexMut`. When `Foo[Bar]` happens, it ends up
/// calling `index_mut`, and therefore, `main` prints `Indexing!`.
///
/// ```
/// use std::ops::{Index, IndexMut};
///
/// #[derive(Copy, Clone)]
/// struct Foo;
/// struct Bar;
///
/// impl Index<Bar> for Foo {
///     type Output = Foo;
///
///     fn index<'a>(&'a self, _index: Bar) -> &'a Foo {
///         self
///     }
/// }
///
/// impl IndexMut<Bar> for Foo {
///     fn index_mut<'a>(&'a mut self, _index: Bar) -> &'a mut Foo {
///         println!("Indexing!");
///         self
///     }
/// }
///
/// fn main() {
///     &mut Foo[Bar];
/// }
/// ```
#[lang="index_mut"]
#[rustc_on_unimplemented = "the type `{Self}` cannot be mutably indexed by `{Idx}`"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait IndexMut<Idx: ?Sized>: Index<Idx> {
    /// The method for the indexing (`Foo[Bar]`) operation
    #[stable(feature = "rust1", since = "1.0.0")]
    fn index_mut<'a>(&'a mut self, index: Idx) -> &'a mut Self::Output;
}

/// An unbounded range.
#[derive(Copy, Clone, PartialEq, Eq)]
#[lang="range_full"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct RangeFull;

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for RangeFull {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "..")
    }
}

/// A (half-open) range which is bounded at both ends.
#[derive(Clone, PartialEq, Eq)]
#[lang="range"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Range<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "rust1", since = "1.0.0")]
    pub start: Idx,
    /// The upper bound of the range (exclusive).
    #[stable(feature = "rust1", since = "1.0.0")]
    pub end: Idx,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<Idx: fmt::Debug> fmt::Debug for Range<Idx> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}..{:?}", self.start, self.end)
    }
}

/// A range which is only bounded below.
#[derive(Clone, PartialEq, Eq)]
#[lang="range_from"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct RangeFrom<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "rust1", since = "1.0.0")]
    pub start: Idx,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<Idx: fmt::Debug> fmt::Debug for RangeFrom<Idx> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}..", self.start)
    }
}

/// A range which is only bounded above.
#[derive(Copy, Clone, PartialEq, Eq)]
#[lang="range_to"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct RangeTo<Idx> {
    /// The upper bound of the range (exclusive).
    #[stable(feature = "rust1", since = "1.0.0")]
    pub end: Idx,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<Idx: fmt::Debug> fmt::Debug for RangeTo<Idx> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "..{:?}", self.end)
    }
}

/// The `Deref` trait is used to specify the functionality of dereferencing
/// operations like `*v`.
///
/// # Examples
///
/// A struct with a single field which is accessible via dereferencing the
/// struct.
///
/// ```
/// use std::ops::Deref;
///
/// struct DerefExample<T> {
///     value: T
/// }
///
/// impl<T> Deref for DerefExample<T> {
///     type Target = T;
///
///     fn deref<'a>(&'a self) -> &'a T {
///         &self.value
///     }
/// }
///
/// fn main() {
///     let x = DerefExample { value: 'a' };
///     assert_eq!('a', *x);
/// }
/// ```
#[lang="deref"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Deref {
    /// The resulting type after dereferencing
    #[stable(feature = "rust1", since = "1.0.0")]
    type Target: ?Sized;

    /// The method called to dereference a value
    #[stable(feature = "rust1", since = "1.0.0")]
    fn deref<'a>(&'a self) -> &'a Self::Target;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T: ?Sized> Deref for &'a T {
    type Target = T;

    fn deref(&self) -> &T { *self }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T: ?Sized> Deref for &'a mut T {
    type Target = T;

    fn deref(&self) -> &T { *self }
}

/// The `DerefMut` trait is used to specify the functionality of dereferencing
/// mutably like `*v = 1;`
///
/// # Examples
///
/// A struct with a single field which is modifiable via dereferencing the
/// struct.
///
/// ```
/// use std::ops::{Deref, DerefMut};
///
/// struct DerefMutExample<T> {
///     value: T
/// }
///
/// impl<T> Deref for DerefMutExample<T> {
///     type Target = T;
///
///     fn deref<'a>(&'a self) -> &'a T {
///         &self.value
///     }
/// }
///
/// impl<T> DerefMut for DerefMutExample<T> {
///     fn deref_mut<'a>(&'a mut self) -> &'a mut T {
///         &mut self.value
///     }
/// }
///
/// fn main() {
///     let mut x = DerefMutExample { value: 'a' };
///     *x = 'b';
///     assert_eq!('b', *x);
/// }
/// ```
#[lang="deref_mut"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait DerefMut: Deref {
    /// The method called to mutably dereference a value
    #[stable(feature = "rust1", since = "1.0.0")]
    fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T: ?Sized> DerefMut for &'a mut T {
    fn deref_mut(&mut self) -> &mut T { *self }
}

/// A version of the call operator that takes an immutable receiver.
#[lang="fn"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[fundamental] // so that regex can rely that `&str: !FnMut`
pub trait Fn<Args> : FnMut<Args> {
    /// This is called when the call operator is used.
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}

/// A version of the call operator that takes a mutable receiver.
#[lang="fn_mut"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[fundamental] // so that regex can rely that `&str: !FnMut`
pub trait FnMut<Args> : FnOnce<Args> {
    /// This is called when the call operator is used.
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}

/// A version of the call operator that takes a by-value receiver.
#[lang="fn_once"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[fundamental] // so that regex can rely that `&str: !FnMut`
pub trait FnOnce<Args> {
    /// The returned type after the call operator is used.
    type Output;

    /// This is called when the call operator is used.
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}

mod impls {
    use marker::Sized;
    use super::{Fn, FnMut, FnOnce};

    impl<'a,A,F:?Sized> Fn<A> for &'a F
        where F : Fn<A>
    {
        extern "rust-call" fn call(&self, args: A) -> F::Output {
            (**self).call(args)
        }
    }

    impl<'a,A,F:?Sized> FnMut<A> for &'a F
        where F : Fn<A>
    {
        extern "rust-call" fn call_mut(&mut self, args: A) -> F::Output {
            (**self).call(args)
        }
    }

    impl<'a,A,F:?Sized> FnOnce<A> for &'a F
        where F : Fn<A>
    {
        type Output = F::Output;

        extern "rust-call" fn call_once(self, args: A) -> F::Output {
            (*self).call(args)
        }
    }

    impl<'a,A,F:?Sized> FnMut<A> for &'a mut F
        where F : FnMut<A>
    {
        extern "rust-call" fn call_mut(&mut self, args: A) -> F::Output {
            (*self).call_mut(args)
        }
    }

    impl<'a,A,F:?Sized> FnOnce<A> for &'a mut F
        where F : FnMut<A>
    {
        type Output = F::Output;
        extern "rust-call" fn call_once(mut self, args: A) -> F::Output {
            (*self).call_mut(args)
        }
    }
}
