// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc(hidden)]

macro_rules! int_module { ($T:ty, $bits:expr) => (

// FIXME(#11621): Should be deprecated once CTFE is implemented in favour of
// calling the `mem::size_of` function.
#[unstable(feature = "core")]
pub const BITS : usize = $bits;
// FIXME(#11621): Should be deprecated once CTFE is implemented in favour of
// calling the `mem::size_of` function.
#[unstable(feature = "core")]
pub const BYTES : usize = ($bits / 8);

// FIXME(#11621): Should be deprecated once CTFE is implemented in favour of
// calling the `Bounded::min_value` function.
#[stable(feature = "rust1", since = "1.0.0")]
pub const MIN: $T = (-1 as $T) << (BITS - 1);
// FIXME(#9837): Compute MIN like this so the high bits that shouldn't exist are 0.
// FIXME(#11621): Should be deprecated once CTFE is implemented in favour of
// calling the `Bounded::max_value` function.
#[stable(feature = "rust1", since = "1.0.0")]
pub const MAX: $T = !MIN;

) }
