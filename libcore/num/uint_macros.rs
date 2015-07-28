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

macro_rules! uint_module { ($T:ty, $T_SIGNED:ty, $bits:expr) => (

#[unstable(feature = "num_bits_bytes",
           reason = "may want to be an associated function")]
#[allow(missing_docs)]
pub const BITS : usize = $bits;
#[unstable(feature = "num_bits_bytes",
           reason = "may want to be an associated function")]
#[allow(missing_docs)]
pub const BYTES : usize = ($bits / 8);

#[stable(feature = "rust1", since = "1.0.0")]
#[allow(missing_docs)]
pub const MIN: $T = 0 as $T;
#[stable(feature = "rust1", since = "1.0.0")]
#[allow(missing_docs)]
pub const MAX: $T = !0 as $T;

) }
