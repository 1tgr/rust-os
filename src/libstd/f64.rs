// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module provides constants which are specific to the implementation
//! of the `f64` floating point data type. Mathematically significant
//! numbers are provided in the `consts` sub-module.
//!
//! *[See also the `f64` primitive type](../primitive.f64.html).*

#![stable(feature = "rust1", since = "1.0.0")]
#![allow(missing_docs)]

#[cfg(not(test))]
use core::num;
#[cfg(not(test))]
use intrinsics;
#[cfg(not(test))]
use num::FpCategory;

#[stable(feature = "rust1", since = "1.0.0")]
pub use core::f64::{RADIX, MANTISSA_DIGITS, DIGITS, EPSILON};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::f64::{MIN_EXP, MAX_EXP, MIN_10_EXP};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::f64::{MAX_10_EXP, NAN, INFINITY, NEG_INFINITY};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::f64::{MIN, MIN_POSITIVE, MAX};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::f64::consts;

#[allow(dead_code)]
mod cmath {
    use libc::{c_double, c_int};

    #[link_name = "m"]
    extern {
        pub fn acos(n: c_double) -> c_double;
        pub fn asin(n: c_double) -> c_double;
        pub fn atan(n: c_double) -> c_double;
        pub fn atan2(a: c_double, b: c_double) -> c_double;
        pub fn cbrt(n: c_double) -> c_double;
        pub fn cosh(n: c_double) -> c_double;
        pub fn erf(n: c_double) -> c_double;
        pub fn erfc(n: c_double) -> c_double;
        pub fn expm1(n: c_double) -> c_double;
        pub fn fdim(a: c_double, b: c_double) -> c_double;
        pub fn fmax(a: c_double, b: c_double) -> c_double;
        pub fn fmin(a: c_double, b: c_double) -> c_double;
        pub fn fmod(a: c_double, b: c_double) -> c_double;
        pub fn frexp(n: c_double, value: &mut c_int) -> c_double;
        pub fn ilogb(n: c_double) -> c_int;
        pub fn ldexp(x: c_double, n: c_int) -> c_double;
        pub fn logb(n: c_double) -> c_double;
        pub fn log1p(n: c_double) -> c_double;
        pub fn nextafter(x: c_double, y: c_double) -> c_double;
        pub fn modf(n: c_double, iptr: &mut c_double) -> c_double;
        pub fn sinh(n: c_double) -> c_double;
        pub fn tan(n: c_double) -> c_double;
        pub fn tanh(n: c_double) -> c_double;
        pub fn tgamma(n: c_double) -> c_double;

        // These are commonly only available for doubles

        pub fn j0(n: c_double) -> c_double;
        pub fn j1(n: c_double) -> c_double;
        pub fn jn(i: c_int, n: c_double) -> c_double;

        pub fn y0(n: c_double) -> c_double;
        pub fn y1(n: c_double) -> c_double;
        pub fn yn(i: c_int, n: c_double) -> c_double;

        #[cfg_attr(all(windows, target_env = "msvc"), link_name = "__lgamma_r")]
        pub fn lgamma_r(n: c_double, sign: &mut c_int) -> c_double;

        #[cfg_attr(all(windows, target_env = "msvc"), link_name = "_hypot")]
        pub fn hypot(x: c_double, y: c_double) -> c_double;
    }
}

#[cfg(not(test))]
#[lang = "f64"]
impl f64 {
    /// Returns `true` if this value is `NaN` and false otherwise.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let nan = f64::NAN;
    /// let f = 7.0_f64;
    ///
    /// assert!(nan.is_nan());
    /// assert!(!f.is_nan());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_nan(self) -> bool { num::Float::is_nan(self) }

    /// Returns `true` if this value is positive infinity or negative infinity and
    /// false otherwise.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = 7.0f64;
    /// let inf = f64::INFINITY;
    /// let neg_inf = f64::NEG_INFINITY;
    /// let nan = f64::NAN;
    ///
    /// assert!(!f.is_infinite());
    /// assert!(!nan.is_infinite());
    ///
    /// assert!(inf.is_infinite());
    /// assert!(neg_inf.is_infinite());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_infinite(self) -> bool { num::Float::is_infinite(self) }

    /// Returns `true` if this number is neither infinite nor `NaN`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = 7.0f64;
    /// let inf: f64 = f64::INFINITY;
    /// let neg_inf: f64 = f64::NEG_INFINITY;
    /// let nan: f64 = f64::NAN;
    ///
    /// assert!(f.is_finite());
    ///
    /// assert!(!nan.is_finite());
    /// assert!(!inf.is_finite());
    /// assert!(!neg_inf.is_finite());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_finite(self) -> bool { num::Float::is_finite(self) }

    /// Returns `true` if the number is neither zero, infinite,
    /// [subnormal][subnormal], or `NaN`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let min = f64::MIN_POSITIVE; // 2.2250738585072014e-308f64
    /// let max = f64::MAX;
    /// let lower_than_min = 1.0e-308_f64;
    /// let zero = 0.0f64;
    ///
    /// assert!(min.is_normal());
    /// assert!(max.is_normal());
    ///
    /// assert!(!zero.is_normal());
    /// assert!(!f64::NAN.is_normal());
    /// assert!(!f64::INFINITY.is_normal());
    /// // Values between `0` and `min` are Subnormal.
    /// assert!(!lower_than_min.is_normal());
    /// ```
    /// [subnormal]: https://en.wikipedia.org/wiki/Denormal_number
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_normal(self) -> bool { num::Float::is_normal(self) }

    /// Returns the floating point category of the number. If only one property
    /// is going to be tested, it is generally faster to use the specific
    /// predicate instead.
    ///
    /// ```
    /// use std::num::FpCategory;
    /// use std::f64;
    ///
    /// let num = 12.4_f64;
    /// let inf = f64::INFINITY;
    ///
    /// assert_eq!(num.classify(), FpCategory::Normal);
    /// assert_eq!(inf.classify(), FpCategory::Infinite);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn classify(self) -> FpCategory { num::Float::classify(self) }

    /// Returns the largest integer less than or equal to a number.
    ///
    /// ```
    /// let f = 3.99_f64;
    /// let g = 3.0_f64;
    ///
    /// assert_eq!(f.floor(), 3.0);
    /// assert_eq!(g.floor(), 3.0);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn floor(self) -> f64 {
        unsafe { intrinsics::floorf64(self) }
    }

    /// Returns the smallest integer greater than or equal to a number.
    ///
    /// ```
    /// let f = 3.01_f64;
    /// let g = 4.0_f64;
    ///
    /// assert_eq!(f.ceil(), 4.0);
    /// assert_eq!(g.ceil(), 4.0);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn ceil(self) -> f64 {
        unsafe { intrinsics::ceilf64(self) }
    }

    /// Returns the nearest integer to a number. Round half-way cases away from
    /// `0.0`.
    ///
    /// ```
    /// let f = 3.3_f64;
    /// let g = -3.3_f64;
    ///
    /// assert_eq!(f.round(), 3.0);
    /// assert_eq!(g.round(), -3.0);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn round(self) -> f64 {
        unsafe { intrinsics::roundf64(self) }
    }

    /// Returns the integer part of a number.
    ///
    /// ```
    /// let f = 3.3_f64;
    /// let g = -3.7_f64;
    ///
    /// assert_eq!(f.trunc(), 3.0);
    /// assert_eq!(g.trunc(), -3.0);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn trunc(self) -> f64 {
        unsafe { intrinsics::truncf64(self) }
    }

    /// Returns the fractional part of a number.
    ///
    /// ```
    /// let x = 3.5_f64;
    /// let y = -3.5_f64;
    /// let abs_difference_x = (x.fract() - 0.5).abs();
    /// let abs_difference_y = (y.fract() - (-0.5)).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn fract(self) -> f64 { self - self.trunc() }

    /// Computes the absolute value of `self`. Returns `NAN` if the
    /// number is `NAN`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = 3.5_f64;
    /// let y = -3.5_f64;
    ///
    /// let abs_difference_x = (x.abs() - x).abs();
    /// let abs_difference_y = (y.abs() - (-y)).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    ///
    /// assert!(f64::NAN.abs().is_nan());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn abs(self) -> f64 { num::Float::abs(self) }

    /// Returns a number that represents the sign of `self`.
    ///
    /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
    /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
    /// - `NAN` if the number is `NAN`
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = 3.5_f64;
    ///
    /// assert_eq!(f.signum(), 1.0);
    /// assert_eq!(f64::NEG_INFINITY.signum(), -1.0);
    ///
    /// assert!(f64::NAN.signum().is_nan());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn signum(self) -> f64 { num::Float::signum(self) }

    /// Returns `true` if `self`'s sign bit is positive, including
    /// `+0.0` and `INFINITY`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let nan: f64 = f64::NAN;
    ///
    /// let f = 7.0_f64;
    /// let g = -7.0_f64;
    ///
    /// assert!(f.is_sign_positive());
    /// assert!(!g.is_sign_positive());
    /// // Requires both tests to determine if is `NaN`
    /// assert!(!nan.is_sign_positive() && !nan.is_sign_negative());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_sign_positive(self) -> bool { num::Float::is_sign_positive(self) }

    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_deprecated(since = "1.0.0", reason = "renamed to is_sign_positive")]
    #[inline]
    pub fn is_positive(self) -> bool { num::Float::is_sign_positive(self) }

    /// Returns `true` if `self`'s sign is negative, including `-0.0`
    /// and `NEG_INFINITY`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let nan = f64::NAN;
    ///
    /// let f = 7.0_f64;
    /// let g = -7.0_f64;
    ///
    /// assert!(!f.is_sign_negative());
    /// assert!(g.is_sign_negative());
    /// // Requires both tests to determine if is `NaN`.
    /// assert!(!nan.is_sign_positive() && !nan.is_sign_negative());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_sign_negative(self) -> bool { num::Float::is_sign_negative(self) }

    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_deprecated(since = "1.0.0", reason = "renamed to is_sign_negative")]
    #[inline]
    pub fn is_negative(self) -> bool { num::Float::is_sign_negative(self) }

    /// Fused multiply-add. Computes `(self * a) + b` with only one rounding
    /// error. This produces a more accurate result with better performance than
    /// a separate multiplication operation followed by an add.
    ///
    /// ```
    /// let m = 10.0_f64;
    /// let x = 4.0_f64;
    /// let b = 60.0_f64;
    ///
    /// // 100.0
    /// let abs_difference = (m.mul_add(x, b) - (m*x + b)).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn mul_add(self, a: f64, b: f64) -> f64 {
        unsafe { intrinsics::fmaf64(self, a, b) }
    }

    /// Takes the reciprocal (inverse) of a number, `1/x`.
    ///
    /// ```
    /// let x = 2.0_f64;
    /// let abs_difference = (x.recip() - (1.0/x)).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn recip(self) -> f64 { num::Float::recip(self) }

    /// Raises a number to an integer power.
    ///
    /// Using this function is generally faster than using `powf`
    ///
    /// ```
    /// let x = 2.0_f64;
    /// let abs_difference = (x.powi(2) - x*x).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn powi(self, n: i32) -> f64 { num::Float::powi(self, n) }

    /// Raises a number to a floating point power.
    ///
    /// ```
    /// let x = 2.0_f64;
    /// let abs_difference = (x.powf(2.0) - x*x).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn powf(self, n: f64) -> f64 {
        unsafe { intrinsics::powf64(self, n) }
    }

    /// Takes the square root of a number.
    ///
    /// Returns NaN if `self` is a negative number.
    ///
    /// ```
    /// let positive = 4.0_f64;
    /// let negative = -4.0_f64;
    ///
    /// let abs_difference = (positive.sqrt() - 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// assert!(negative.sqrt().is_nan());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn sqrt(self) -> f64 {
        if self < 0.0 {
            NAN
        } else {
            unsafe { intrinsics::sqrtf64(self) }
        }
    }

    /// Returns `e^(self)`, (the exponential function).
    ///
    /// ```
    /// let one = 1.0_f64;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn exp(self) -> f64 {
        unsafe { intrinsics::expf64(self) }
    }

    /// Returns `2^(self)`.
    ///
    /// ```
    /// let f = 2.0_f64;
    ///
    /// // 2^2 - 4 == 0
    /// let abs_difference = (f.exp2() - 4.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn exp2(self) -> f64 {
        unsafe { intrinsics::exp2f64(self) }
    }

    /// Returns the natural logarithm of the number.
    ///
    /// ```
    /// let one = 1.0_f64;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn ln(self) -> f64 {
        self.log_wrapper(|n| { unsafe { intrinsics::logf64(n) } })
    }

    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// ```
    /// let ten = 10.0_f64;
    /// let two = 2.0_f64;
    ///
    /// // log10(10) - 1 == 0
    /// let abs_difference_10 = (ten.log(10.0) - 1.0).abs();
    ///
    /// // log2(2) - 1 == 0
    /// let abs_difference_2 = (two.log(2.0) - 1.0).abs();
    ///
    /// assert!(abs_difference_10 < 1e-10);
    /// assert!(abs_difference_2 < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn log(self, base: f64) -> f64 { self.ln() / base.ln() }

    /// Returns the base 2 logarithm of the number.
    ///
    /// ```
    /// let two = 2.0_f64;
    ///
    /// // log2(2) - 1 == 0
    /// let abs_difference = (two.log2() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn log2(self) -> f64 {
        self.log_wrapper(|n| {
            #[cfg(target_os = "android")]
            return ::sys::android::log2f64(n);
            #[cfg(not(target_os = "android"))]
            return unsafe { intrinsics::log2f64(n) };
        })
    }

    /// Returns the base 10 logarithm of the number.
    ///
    /// ```
    /// let ten = 10.0_f64;
    ///
    /// // log10(10) - 1 == 0
    /// let abs_difference = (ten.log10() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn log10(self) -> f64 {
        self.log_wrapper(|n| { unsafe { intrinsics::log10f64(n) } })
    }

    /// Converts radians to degrees.
    ///
    /// ```
    /// use std::f64::consts;
    ///
    /// let angle = consts::PI;
    ///
    /// let abs_difference = (angle.to_degrees() - 180.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn to_degrees(self) -> f64 { num::Float::to_degrees(self) }

    /// Converts degrees to radians.
    ///
    /// ```
    /// use std::f64::consts;
    ///
    /// let angle = 180.0_f64;
    ///
    /// let abs_difference = (angle.to_radians() - consts::PI).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn to_radians(self) -> f64 { num::Float::to_radians(self) }

    /// Returns the maximum of the two numbers.
    ///
    /// ```
    /// let x = 1.0_f64;
    /// let y = 2.0_f64;
    ///
    /// assert_eq!(x.max(y), y);
    /// ```
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn max(self, other: f64) -> f64 {
        unsafe { cmath::fmax(self, other) }
    }

    /// Returns the minimum of the two numbers.
    ///
    /// ```
    /// let x = 1.0_f64;
    /// let y = 2.0_f64;
    ///
    /// assert_eq!(x.min(y), x);
    /// ```
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn min(self, other: f64) -> f64 {
        unsafe { cmath::fmin(self, other) }
    }

    /// The positive difference of two numbers.
    ///
    /// * If `self <= other`: `0:0`
    /// * Else: `self - other`
    ///
    /// ```
    /// let x = 3.0_f64;
    /// let y = -3.0_f64;
    ///
    /// let abs_difference_x = (x.abs_sub(1.0) - 2.0).abs();
    /// let abs_difference_y = (y.abs_sub(1.0) - 0.0).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    #[rustc_deprecated(since = "1.10.0",
                       reason = "you probably meant `(self - other).abs()`: \
                                 this operation is `(self - other).max(0.0)` (also \
                                 known as `fdim` in C). If you truly need the positive \
                                 difference, consider using that expression or the C function \
                                 `fdim`, depending on how you wish to handle NaN (please consider \
                                 filing an issue describing your use-case too).")]
     pub fn abs_sub(self, other: f64) -> f64 {
         unsafe { cmath::fdim(self, other) }
     }

    /// Takes the cubic root of a number.
    ///
    /// ```
    /// let x = 8.0_f64;
    ///
    /// // x^(1/3) - 2 == 0
    /// let abs_difference = (x.cbrt() - 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn cbrt(self) -> f64 {
        unsafe { cmath::cbrt(self) }
    }

    /// Calculates the length of the hypotenuse of a right-angle triangle given
    /// legs of length `x` and `y`.
    ///
    /// ```
    /// let x = 2.0_f64;
    /// let y = 3.0_f64;
    ///
    /// // sqrt(x^2 + y^2)
    /// let abs_difference = (x.hypot(y) - (x.powi(2) + y.powi(2)).sqrt()).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn hypot(self, other: f64) -> f64 {
        unsafe { cmath::hypot(self, other) }
    }

    /// Computes the sine of a number (in radians).
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = f64::consts::PI/2.0;
    ///
    /// let abs_difference = (x.sin() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn sin(self) -> f64 {
        unsafe { intrinsics::sinf64(self) }
    }

    /// Computes the cosine of a number (in radians).
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = 2.0*f64::consts::PI;
    ///
    /// let abs_difference = (x.cos() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn cos(self) -> f64 {
        unsafe { intrinsics::cosf64(self) }
    }

    /// Computes the tangent of a number (in radians).
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = f64::consts::PI/4.0;
    /// let abs_difference = (x.tan() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-14);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn tan(self) -> f64 {
        unsafe { cmath::tan(self) }
    }

    /// Computes the arcsine of a number. Return value is in radians in
    /// the range [-pi/2, pi/2] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = f64::consts::PI / 2.0;
    ///
    /// // asin(sin(pi/2))
    /// let abs_difference = (f.sin().asin() - f64::consts::PI / 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn asin(self) -> f64 {
        unsafe { cmath::asin(self) }
    }

    /// Computes the arccosine of a number. Return value is in radians in
    /// the range [0, pi] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = f64::consts::PI / 4.0;
    ///
    /// // acos(cos(pi/4))
    /// let abs_difference = (f.cos().acos() - f64::consts::PI / 4.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn acos(self) -> f64 {
        unsafe { cmath::acos(self) }
    }

    /// Computes the arctangent of a number. Return value is in radians in the
    /// range [-pi/2, pi/2];
    ///
    /// ```
    /// let f = 1.0_f64;
    ///
    /// // atan(tan(1))
    /// let abs_difference = (f.tan().atan() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn atan(self) -> f64 {
        unsafe { cmath::atan(self) }
    }

    /// Computes the four quadrant arctangent of `self` (`y`) and `other` (`x`).
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
    /// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
    /// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
    ///
    /// ```
    /// use std::f64;
    ///
    /// let pi = f64::consts::PI;
    /// // All angles from horizontal right (+x)
    /// // 45 deg counter-clockwise
    /// let x1 = 3.0_f64;
    /// let y1 = -3.0_f64;
    ///
    /// // 135 deg clockwise
    /// let x2 = -3.0_f64;
    /// let y2 = 3.0_f64;
    ///
    /// let abs_difference_1 = (y1.atan2(x1) - (-pi/4.0)).abs();
    /// let abs_difference_2 = (y2.atan2(x2) - 3.0*pi/4.0).abs();
    ///
    /// assert!(abs_difference_1 < 1e-10);
    /// assert!(abs_difference_2 < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn atan2(self, other: f64) -> f64 {
        unsafe { cmath::atan2(self, other) }
    }

    /// Simultaneously computes the sine and cosine of the number, `x`. Returns
    /// `(sin(x), cos(x))`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = f64::consts::PI/4.0;
    /// let f = x.sin_cos();
    ///
    /// let abs_difference_0 = (f.0 - x.sin()).abs();
    /// let abs_difference_1 = (f.1 - x.cos()).abs();
    ///
    /// assert!(abs_difference_0 < 1e-10);
    /// assert!(abs_difference_1 < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn sin_cos(self) -> (f64, f64) {
        (self.sin(), self.cos())
    }

    /// Returns `e^(self) - 1` in a way that is accurate even if the
    /// number is close to zero.
    ///
    /// ```
    /// let x = 7.0_f64;
    ///
    /// // e^(ln(7)) - 1
    /// let abs_difference = (x.ln().exp_m1() - 6.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn exp_m1(self) -> f64 {
        unsafe { cmath::expm1(self) }
    }

    /// Returns `ln(1+n)` (natural logarithm) more accurately than if
    /// the operations were performed separately.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = f64::consts::E - 1.0;
    ///
    /// // ln(1 + (e - 1)) == ln(e) == 1
    /// let abs_difference = (x.ln_1p() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn ln_1p(self) -> f64 {
        unsafe { cmath::log1p(self) }
    }

    /// Hyperbolic sine function.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0_f64;
    ///
    /// let f = x.sinh();
    /// // Solving sinh() at 1 gives `(e^2-1)/(2e)`
    /// let g = (e*e - 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn sinh(self) -> f64 {
        unsafe { cmath::sinh(self) }
    }

    /// Hyperbolic cosine function.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0_f64;
    /// let f = x.cosh();
    /// // Solving cosh() at 1 gives this result
    /// let g = (e*e + 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// // Same result
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn cosh(self) -> f64 {
        unsafe { cmath::cosh(self) }
    }

    /// Hyperbolic tangent function.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0_f64;
    ///
    /// let f = x.tanh();
    /// // Solving tanh() at 1 gives `(1 - e^(-2))/(1 + e^(-2))`
    /// let g = (1.0 - e.powi(-2))/(1.0 + e.powi(-2));
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn tanh(self) -> f64 {
        unsafe { cmath::tanh(self) }
    }

    /// Inverse hyperbolic sine function.
    ///
    /// ```
    /// let x = 1.0_f64;
    /// let f = x.sinh().asinh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn asinh(self) -> f64 {
        if self == NEG_INFINITY {
            NEG_INFINITY
        } else {
            (self + ((self * self) + 1.0).sqrt()).ln()
        }
    }

    /// Inverse hyperbolic cosine function.
    ///
    /// ```
    /// let x = 1.0_f64;
    /// let f = x.cosh().acosh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn acosh(self) -> f64 {
        match self {
            x if x < 1.0 => NAN,
            x => (x + ((x * x) - 1.0).sqrt()).ln(),
        }
    }

    /// Inverse hyperbolic tangent function.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let f = e.tanh().atanh();
    ///
    /// let abs_difference = (f - e).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn atanh(self) -> f64 {
        0.5 * ((2.0 * self) / (1.0 - self)).ln_1p()
    }

    // Solaris/Illumos requires a wrapper around log, log2, and log10 functions
    // because of their non-standard behavior (e.g. log(-n) returns -Inf instead
    // of expected NaN).
    fn log_wrapper<F: Fn(f64) -> f64>(self, log_fn: F) -> f64 {
        if !cfg!(target_os = "solaris") {
            log_fn(self)
        } else {
            if self.is_finite() {
                if self > 0.0 {
                    log_fn(self)
                } else if self == 0.0 {
                    NEG_INFINITY // log(0) = -Inf
                } else {
                    NAN // log(-n) = NaN
                }
            } else if self.is_nan() {
                self // log(NaN) = NaN
            } else if self > 0.0 {
                self // log(Inf) = Inf
            } else {
                NAN // log(-Inf) = NaN
            }
        }
    }

    /// Raw transmutation to `u64`.
    ///
    /// Converts the `f64` into its raw memory representation,
    /// similar to the `transmute` function.
    ///
    /// Note that this function is distinct from casting.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(float_bits_conv)]
    /// assert!((1f64).to_bits() != 1f64 as u64); // to_bits() is not casting!
    /// assert_eq!((12.5f64).to_bits(), 0x4029000000000000);
    ///
    /// ```
    #[unstable(feature = "float_bits_conv", reason = "recently added", issue = "40470")]
    #[inline]
    pub fn to_bits(self) -> u64 {
        unsafe { ::mem::transmute(self) }
    }

    /// Raw transmutation from `u64`.
    ///
    /// Converts the given `u64` containing the float's raw memory
    /// representation into the `f64` type, similar to the
    /// `transmute` function.
    ///
    /// There is only one difference to a bare `transmute`:
    /// Due to the implications onto Rust's safety promises being
    /// uncertain, if the representation of a signaling NaN "sNaN" float
    /// is passed to the function, the implementation is allowed to
    /// return a quiet NaN instead.
    ///
    /// Note that this function is distinct from casting.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(float_bits_conv)]
    /// use std::f64;
    /// let v = f64::from_bits(0x4029000000000000);
    /// let difference = (v - 12.5).abs();
    /// assert!(difference <= 1e-5);
    /// // Example for a signaling NaN value:
    /// let snan = 0x7FF0000000000001;
    /// assert_ne!(f64::from_bits(snan).to_bits(), snan);
    /// ```
    #[unstable(feature = "float_bits_conv", reason = "recently added", issue = "40470")]
    #[inline]
    pub fn from_bits(mut v: u64) -> Self {
        const EXP_MASK: u64   = 0x7FF0000000000000;
        const QNAN_MASK: u64  = 0x0001000000000000;
        const FRACT_MASK: u64 = 0x000FFFFFFFFFFFFF;
        if v & EXP_MASK == EXP_MASK && v & FRACT_MASK != 0 {
            // If we have a NaN value, we
            // convert signaling NaN values to quiet NaN
            // by setting the the highest bit of the fraction
            v |= QNAN_MASK;
        }
        unsafe { ::mem::transmute(v) }
    }
}

#[cfg(test)]
mod tests {
    use f64;
    use f64::*;
    use num::*;
    use num::FpCategory as Fp;

    #[test]
    fn test_num_f64() {
        test_num(10f64, 2f64);
    }

    #[test]
    fn test_min_nan() {
        assert_eq!(NAN.min(2.0), 2.0);
        assert_eq!(2.0f64.min(NAN), 2.0);
    }

    #[test]
    fn test_max_nan() {
        assert_eq!(NAN.max(2.0), 2.0);
        assert_eq!(2.0f64.max(NAN), 2.0);
    }

    #[test]
    fn test_nan() {
        let nan: f64 = NAN;
        assert!(nan.is_nan());
        assert!(!nan.is_infinite());
        assert!(!nan.is_finite());
        assert!(!nan.is_normal());
        assert!(!nan.is_sign_positive());
        assert!(!nan.is_sign_negative());
        assert_eq!(Fp::Nan, nan.classify());
    }

    #[test]
    fn test_infinity() {
        let inf: f64 = INFINITY;
        assert!(inf.is_infinite());
        assert!(!inf.is_finite());
        assert!(inf.is_sign_positive());
        assert!(!inf.is_sign_negative());
        assert!(!inf.is_nan());
        assert!(!inf.is_normal());
        assert_eq!(Fp::Infinite, inf.classify());
    }

    #[test]
    fn test_neg_infinity() {
        let neg_inf: f64 = NEG_INFINITY;
        assert!(neg_inf.is_infinite());
        assert!(!neg_inf.is_finite());
        assert!(!neg_inf.is_sign_positive());
        assert!(neg_inf.is_sign_negative());
        assert!(!neg_inf.is_nan());
        assert!(!neg_inf.is_normal());
        assert_eq!(Fp::Infinite, neg_inf.classify());
    }

    #[test]
    fn test_zero() {
        let zero: f64 = 0.0f64;
        assert_eq!(0.0, zero);
        assert!(!zero.is_infinite());
        assert!(zero.is_finite());
        assert!(zero.is_sign_positive());
        assert!(!zero.is_sign_negative());
        assert!(!zero.is_nan());
        assert!(!zero.is_normal());
        assert_eq!(Fp::Zero, zero.classify());
    }

    #[test]
    fn test_neg_zero() {
        let neg_zero: f64 = -0.0;
        assert_eq!(0.0, neg_zero);
        assert!(!neg_zero.is_infinite());
        assert!(neg_zero.is_finite());
        assert!(!neg_zero.is_sign_positive());
        assert!(neg_zero.is_sign_negative());
        assert!(!neg_zero.is_nan());
        assert!(!neg_zero.is_normal());
        assert_eq!(Fp::Zero, neg_zero.classify());
    }

    #[test]
    fn test_one() {
        let one: f64 = 1.0f64;
        assert_eq!(1.0, one);
        assert!(!one.is_infinite());
        assert!(one.is_finite());
        assert!(one.is_sign_positive());
        assert!(!one.is_sign_negative());
        assert!(!one.is_nan());
        assert!(one.is_normal());
        assert_eq!(Fp::Normal, one.classify());
    }

    #[test]
    fn test_is_nan() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert!(nan.is_nan());
        assert!(!0.0f64.is_nan());
        assert!(!5.3f64.is_nan());
        assert!(!(-10.732f64).is_nan());
        assert!(!inf.is_nan());
        assert!(!neg_inf.is_nan());
    }

    #[test]
    fn test_is_infinite() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert!(!nan.is_infinite());
        assert!(inf.is_infinite());
        assert!(neg_inf.is_infinite());
        assert!(!0.0f64.is_infinite());
        assert!(!42.8f64.is_infinite());
        assert!(!(-109.2f64).is_infinite());
    }

    #[test]
    fn test_is_finite() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert!(!nan.is_finite());
        assert!(!inf.is_finite());
        assert!(!neg_inf.is_finite());
        assert!(0.0f64.is_finite());
        assert!(42.8f64.is_finite());
        assert!((-109.2f64).is_finite());
    }

    #[test]
    fn test_is_normal() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        let zero: f64 = 0.0f64;
        let neg_zero: f64 = -0.0;
        assert!(!nan.is_normal());
        assert!(!inf.is_normal());
        assert!(!neg_inf.is_normal());
        assert!(!zero.is_normal());
        assert!(!neg_zero.is_normal());
        assert!(1f64.is_normal());
        assert!(1e-307f64.is_normal());
        assert!(!1e-308f64.is_normal());
    }

    #[test]
    fn test_classify() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        let zero: f64 = 0.0f64;
        let neg_zero: f64 = -0.0;
        assert_eq!(nan.classify(), Fp::Nan);
        assert_eq!(inf.classify(), Fp::Infinite);
        assert_eq!(neg_inf.classify(), Fp::Infinite);
        assert_eq!(zero.classify(), Fp::Zero);
        assert_eq!(neg_zero.classify(), Fp::Zero);
        assert_eq!(1e-307f64.classify(), Fp::Normal);
        assert_eq!(1e-308f64.classify(), Fp::Subnormal);
    }

    #[test]
    fn test_floor() {
        assert_approx_eq!(1.0f64.floor(), 1.0f64);
        assert_approx_eq!(1.3f64.floor(), 1.0f64);
        assert_approx_eq!(1.5f64.floor(), 1.0f64);
        assert_approx_eq!(1.7f64.floor(), 1.0f64);
        assert_approx_eq!(0.0f64.floor(), 0.0f64);
        assert_approx_eq!((-0.0f64).floor(), -0.0f64);
        assert_approx_eq!((-1.0f64).floor(), -1.0f64);
        assert_approx_eq!((-1.3f64).floor(), -2.0f64);
        assert_approx_eq!((-1.5f64).floor(), -2.0f64);
        assert_approx_eq!((-1.7f64).floor(), -2.0f64);
    }

    #[test]
    fn test_ceil() {
        assert_approx_eq!(1.0f64.ceil(), 1.0f64);
        assert_approx_eq!(1.3f64.ceil(), 2.0f64);
        assert_approx_eq!(1.5f64.ceil(), 2.0f64);
        assert_approx_eq!(1.7f64.ceil(), 2.0f64);
        assert_approx_eq!(0.0f64.ceil(), 0.0f64);
        assert_approx_eq!((-0.0f64).ceil(), -0.0f64);
        assert_approx_eq!((-1.0f64).ceil(), -1.0f64);
        assert_approx_eq!((-1.3f64).ceil(), -1.0f64);
        assert_approx_eq!((-1.5f64).ceil(), -1.0f64);
        assert_approx_eq!((-1.7f64).ceil(), -1.0f64);
    }

    #[test]
    fn test_round() {
        assert_approx_eq!(1.0f64.round(), 1.0f64);
        assert_approx_eq!(1.3f64.round(), 1.0f64);
        assert_approx_eq!(1.5f64.round(), 2.0f64);
        assert_approx_eq!(1.7f64.round(), 2.0f64);
        assert_approx_eq!(0.0f64.round(), 0.0f64);
        assert_approx_eq!((-0.0f64).round(), -0.0f64);
        assert_approx_eq!((-1.0f64).round(), -1.0f64);
        assert_approx_eq!((-1.3f64).round(), -1.0f64);
        assert_approx_eq!((-1.5f64).round(), -2.0f64);
        assert_approx_eq!((-1.7f64).round(), -2.0f64);
    }

    #[test]
    fn test_trunc() {
        assert_approx_eq!(1.0f64.trunc(), 1.0f64);
        assert_approx_eq!(1.3f64.trunc(), 1.0f64);
        assert_approx_eq!(1.5f64.trunc(), 1.0f64);
        assert_approx_eq!(1.7f64.trunc(), 1.0f64);
        assert_approx_eq!(0.0f64.trunc(), 0.0f64);
        assert_approx_eq!((-0.0f64).trunc(), -0.0f64);
        assert_approx_eq!((-1.0f64).trunc(), -1.0f64);
        assert_approx_eq!((-1.3f64).trunc(), -1.0f64);
        assert_approx_eq!((-1.5f64).trunc(), -1.0f64);
        assert_approx_eq!((-1.7f64).trunc(), -1.0f64);
    }

    #[test]
    fn test_fract() {
        assert_approx_eq!(1.0f64.fract(), 0.0f64);
        assert_approx_eq!(1.3f64.fract(), 0.3f64);
        assert_approx_eq!(1.5f64.fract(), 0.5f64);
        assert_approx_eq!(1.7f64.fract(), 0.7f64);
        assert_approx_eq!(0.0f64.fract(), 0.0f64);
        assert_approx_eq!((-0.0f64).fract(), -0.0f64);
        assert_approx_eq!((-1.0f64).fract(), -0.0f64);
        assert_approx_eq!((-1.3f64).fract(), -0.3f64);
        assert_approx_eq!((-1.5f64).fract(), -0.5f64);
        assert_approx_eq!((-1.7f64).fract(), -0.7f64);
    }

    #[test]
    fn test_abs() {
        assert_eq!(INFINITY.abs(), INFINITY);
        assert_eq!(1f64.abs(), 1f64);
        assert_eq!(0f64.abs(), 0f64);
        assert_eq!((-0f64).abs(), 0f64);
        assert_eq!((-1f64).abs(), 1f64);
        assert_eq!(NEG_INFINITY.abs(), INFINITY);
        assert_eq!((1f64/NEG_INFINITY).abs(), 0f64);
        assert!(NAN.abs().is_nan());
    }

    #[test]
    fn test_signum() {
        assert_eq!(INFINITY.signum(), 1f64);
        assert_eq!(1f64.signum(), 1f64);
        assert_eq!(0f64.signum(), 1f64);
        assert_eq!((-0f64).signum(), -1f64);
        assert_eq!((-1f64).signum(), -1f64);
        assert_eq!(NEG_INFINITY.signum(), -1f64);
        assert_eq!((1f64/NEG_INFINITY).signum(), -1f64);
        assert!(NAN.signum().is_nan());
    }

    #[test]
    fn test_is_sign_positive() {
        assert!(INFINITY.is_sign_positive());
        assert!(1f64.is_sign_positive());
        assert!(0f64.is_sign_positive());
        assert!(!(-0f64).is_sign_positive());
        assert!(!(-1f64).is_sign_positive());
        assert!(!NEG_INFINITY.is_sign_positive());
        assert!(!(1f64/NEG_INFINITY).is_sign_positive());
        assert!(!NAN.is_sign_positive());
    }

    #[test]
    fn test_is_sign_negative() {
        assert!(!INFINITY.is_sign_negative());
        assert!(!1f64.is_sign_negative());
        assert!(!0f64.is_sign_negative());
        assert!((-0f64).is_sign_negative());
        assert!((-1f64).is_sign_negative());
        assert!(NEG_INFINITY.is_sign_negative());
        assert!((1f64/NEG_INFINITY).is_sign_negative());
        assert!(!NAN.is_sign_negative());
    }

    #[test]
    fn test_mul_add() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_approx_eq!(12.3f64.mul_add(4.5, 6.7), 62.05);
        assert_approx_eq!((-12.3f64).mul_add(-4.5, -6.7), 48.65);
        assert_approx_eq!(0.0f64.mul_add(8.9, 1.2), 1.2);
        assert_approx_eq!(3.4f64.mul_add(-0.0, 5.6), 5.6);
        assert!(nan.mul_add(7.8, 9.0).is_nan());
        assert_eq!(inf.mul_add(7.8, 9.0), inf);
        assert_eq!(neg_inf.mul_add(7.8, 9.0), neg_inf);
        assert_eq!(8.9f64.mul_add(inf, 3.2), inf);
        assert_eq!((-3.2f64).mul_add(2.4, neg_inf), neg_inf);
    }

    #[test]
    fn test_recip() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_eq!(1.0f64.recip(), 1.0);
        assert_eq!(2.0f64.recip(), 0.5);
        assert_eq!((-0.4f64).recip(), -2.5);
        assert_eq!(0.0f64.recip(), inf);
        assert!(nan.recip().is_nan());
        assert_eq!(inf.recip(), 0.0);
        assert_eq!(neg_inf.recip(), 0.0);
    }

    #[test]
    fn test_powi() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_eq!(1.0f64.powi(1), 1.0);
        assert_approx_eq!((-3.1f64).powi(2), 9.61);
        assert_approx_eq!(5.9f64.powi(-2), 0.028727);
        assert_eq!(8.3f64.powi(0), 1.0);
        assert!(nan.powi(2).is_nan());
        assert_eq!(inf.powi(3), inf);
        assert_eq!(neg_inf.powi(2), inf);
    }

    #[test]
    fn test_powf() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_eq!(1.0f64.powf(1.0), 1.0);
        assert_approx_eq!(3.4f64.powf(4.5), 246.408183);
        assert_approx_eq!(2.7f64.powf(-3.2), 0.041652);
        assert_approx_eq!((-3.1f64).powf(2.0), 9.61);
        assert_approx_eq!(5.9f64.powf(-2.0), 0.028727);
        assert_eq!(8.3f64.powf(0.0), 1.0);
        assert!(nan.powf(2.0).is_nan());
        assert_eq!(inf.powf(2.0), inf);
        assert_eq!(neg_inf.powf(3.0), neg_inf);
    }

    #[test]
    fn test_sqrt_domain() {
        assert!(NAN.sqrt().is_nan());
        assert!(NEG_INFINITY.sqrt().is_nan());
        assert!((-1.0f64).sqrt().is_nan());
        assert_eq!((-0.0f64).sqrt(), -0.0);
        assert_eq!(0.0f64.sqrt(), 0.0);
        assert_eq!(1.0f64.sqrt(), 1.0);
        assert_eq!(INFINITY.sqrt(), INFINITY);
    }

    #[test]
    fn test_exp() {
        assert_eq!(1.0, 0.0f64.exp());
        assert_approx_eq!(2.718282, 1.0f64.exp());
        assert_approx_eq!(148.413159, 5.0f64.exp());

        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        let nan: f64 = NAN;
        assert_eq!(inf, inf.exp());
        assert_eq!(0.0, neg_inf.exp());
        assert!(nan.exp().is_nan());
    }

    #[test]
    fn test_exp2() {
        assert_eq!(32.0, 5.0f64.exp2());
        assert_eq!(1.0, 0.0f64.exp2());

        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        let nan: f64 = NAN;
        assert_eq!(inf, inf.exp2());
        assert_eq!(0.0, neg_inf.exp2());
        assert!(nan.exp2().is_nan());
    }

    #[test]
    fn test_ln() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_approx_eq!(1.0f64.exp().ln(), 1.0);
        assert!(nan.ln().is_nan());
        assert_eq!(inf.ln(), inf);
        assert!(neg_inf.ln().is_nan());
        assert!((-2.3f64).ln().is_nan());
        assert_eq!((-0.0f64).ln(), neg_inf);
        assert_eq!(0.0f64.ln(), neg_inf);
        assert_approx_eq!(4.0f64.ln(), 1.386294);
    }

    #[test]
    fn test_log() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_eq!(10.0f64.log(10.0), 1.0);
        assert_approx_eq!(2.3f64.log(3.5), 0.664858);
        assert_eq!(1.0f64.exp().log(1.0f64.exp()), 1.0);
        assert!(1.0f64.log(1.0).is_nan());
        assert!(1.0f64.log(-13.9).is_nan());
        assert!(nan.log(2.3).is_nan());
        assert_eq!(inf.log(10.0), inf);
        assert!(neg_inf.log(8.8).is_nan());
        assert!((-2.3f64).log(0.1).is_nan());
        assert_eq!((-0.0f64).log(2.0), neg_inf);
        assert_eq!(0.0f64.log(7.0), neg_inf);
    }

    #[test]
    fn test_log2() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_approx_eq!(10.0f64.log2(), 3.321928);
        assert_approx_eq!(2.3f64.log2(), 1.201634);
        assert_approx_eq!(1.0f64.exp().log2(), 1.442695);
        assert!(nan.log2().is_nan());
        assert_eq!(inf.log2(), inf);
        assert!(neg_inf.log2().is_nan());
        assert!((-2.3f64).log2().is_nan());
        assert_eq!((-0.0f64).log2(), neg_inf);
        assert_eq!(0.0f64.log2(), neg_inf);
    }

    #[test]
    fn test_log10() {
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_eq!(10.0f64.log10(), 1.0);
        assert_approx_eq!(2.3f64.log10(), 0.361728);
        assert_approx_eq!(1.0f64.exp().log10(), 0.434294);
        assert_eq!(1.0f64.log10(), 0.0);
        assert!(nan.log10().is_nan());
        assert_eq!(inf.log10(), inf);
        assert!(neg_inf.log10().is_nan());
        assert!((-2.3f64).log10().is_nan());
        assert_eq!((-0.0f64).log10(), neg_inf);
        assert_eq!(0.0f64.log10(), neg_inf);
    }

    #[test]
    fn test_to_degrees() {
        let pi: f64 = consts::PI;
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_eq!(0.0f64.to_degrees(), 0.0);
        assert_approx_eq!((-5.8f64).to_degrees(), -332.315521);
        assert_eq!(pi.to_degrees(), 180.0);
        assert!(nan.to_degrees().is_nan());
        assert_eq!(inf.to_degrees(), inf);
        assert_eq!(neg_inf.to_degrees(), neg_inf);
    }

    #[test]
    fn test_to_radians() {
        let pi: f64 = consts::PI;
        let nan: f64 = NAN;
        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        assert_eq!(0.0f64.to_radians(), 0.0);
        assert_approx_eq!(154.6f64.to_radians(), 2.698279);
        assert_approx_eq!((-332.31f64).to_radians(), -5.799903);
        assert_eq!(180.0f64.to_radians(), pi);
        assert!(nan.to_radians().is_nan());
        assert_eq!(inf.to_radians(), inf);
        assert_eq!(neg_inf.to_radians(), neg_inf);
    }

    #[test]
    fn test_asinh() {
        assert_eq!(0.0f64.asinh(), 0.0f64);
        assert_eq!((-0.0f64).asinh(), -0.0f64);

        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        let nan: f64 = NAN;
        assert_eq!(inf.asinh(), inf);
        assert_eq!(neg_inf.asinh(), neg_inf);
        assert!(nan.asinh().is_nan());
        assert_approx_eq!(2.0f64.asinh(), 1.443635475178810342493276740273105f64);
        assert_approx_eq!((-2.0f64).asinh(), -1.443635475178810342493276740273105f64);
    }

    #[test]
    fn test_acosh() {
        assert_eq!(1.0f64.acosh(), 0.0f64);
        assert!(0.999f64.acosh().is_nan());

        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        let nan: f64 = NAN;
        assert_eq!(inf.acosh(), inf);
        assert!(neg_inf.acosh().is_nan());
        assert!(nan.acosh().is_nan());
        assert_approx_eq!(2.0f64.acosh(), 1.31695789692481670862504634730796844f64);
        assert_approx_eq!(3.0f64.acosh(), 1.76274717403908605046521864995958461f64);
    }

    #[test]
    fn test_atanh() {
        assert_eq!(0.0f64.atanh(), 0.0f64);
        assert_eq!((-0.0f64).atanh(), -0.0f64);

        let inf: f64 = INFINITY;
        let neg_inf: f64 = NEG_INFINITY;
        let nan: f64 = NAN;
        assert_eq!(1.0f64.atanh(), inf);
        assert_eq!((-1.0f64).atanh(), neg_inf);
        assert!(2f64.atanh().atanh().is_nan());
        assert!((-2f64).atanh().atanh().is_nan());
        assert!(inf.atanh().is_nan());
        assert!(neg_inf.atanh().is_nan());
        assert!(nan.atanh().is_nan());
        assert_approx_eq!(0.5f64.atanh(), 0.54930614433405484569762261846126285f64);
        assert_approx_eq!((-0.5f64).atanh(), -0.54930614433405484569762261846126285f64);
    }

    #[test]
    fn test_real_consts() {
        use super::consts;
        let pi: f64 = consts::PI;
        let frac_pi_2: f64 = consts::FRAC_PI_2;
        let frac_pi_3: f64 = consts::FRAC_PI_3;
        let frac_pi_4: f64 = consts::FRAC_PI_4;
        let frac_pi_6: f64 = consts::FRAC_PI_6;
        let frac_pi_8: f64 = consts::FRAC_PI_8;
        let frac_1_pi: f64 = consts::FRAC_1_PI;
        let frac_2_pi: f64 = consts::FRAC_2_PI;
        let frac_2_sqrtpi: f64 = consts::FRAC_2_SQRT_PI;
        let sqrt2: f64 = consts::SQRT_2;
        let frac_1_sqrt2: f64 = consts::FRAC_1_SQRT_2;
        let e: f64 = consts::E;
        let log2_e: f64 = consts::LOG2_E;
        let log10_e: f64 = consts::LOG10_E;
        let ln_2: f64 = consts::LN_2;
        let ln_10: f64 = consts::LN_10;

        assert_approx_eq!(frac_pi_2, pi / 2f64);
        assert_approx_eq!(frac_pi_3, pi / 3f64);
        assert_approx_eq!(frac_pi_4, pi / 4f64);
        assert_approx_eq!(frac_pi_6, pi / 6f64);
        assert_approx_eq!(frac_pi_8, pi / 8f64);
        assert_approx_eq!(frac_1_pi, 1f64 / pi);
        assert_approx_eq!(frac_2_pi, 2f64 / pi);
        assert_approx_eq!(frac_2_sqrtpi, 2f64 / pi.sqrt());
        assert_approx_eq!(sqrt2, 2f64.sqrt());
        assert_approx_eq!(frac_1_sqrt2, 1f64 / 2f64.sqrt());
        assert_approx_eq!(log2_e, e.log2());
        assert_approx_eq!(log10_e, e.log10());
        assert_approx_eq!(ln_2, 2f64.ln());
        assert_approx_eq!(ln_10, 10f64.ln());
    }

    #[test]
    fn test_float_bits_conv() {
        assert_eq!((1f64).to_bits(), 0x3ff0000000000000);
        assert_eq!((12.5f64).to_bits(), 0x4029000000000000);
        assert_eq!((1337f64).to_bits(), 0x4094e40000000000);
        assert_eq!((-14.25f64).to_bits(), 0xc02c800000000000);
        assert_approx_eq!(f64::from_bits(0x3ff0000000000000), 1.0);
        assert_approx_eq!(f64::from_bits(0x4029000000000000), 12.5);
        assert_approx_eq!(f64::from_bits(0x4094e40000000000), 1337.0);
        assert_approx_eq!(f64::from_bits(0xc02c800000000000), -14.25);
    }
}
