#![no_std]

extern crate alloc;

#[cfg(not(target_os = "rust_os"))]
extern crate cratesio_libc as libc;

#[link(name = "c")]
#[link(name = "freetype")]
#[cfg_attr(target_os = "rust_os", link(name = "gcc"))]
extern "C" {}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod bindings;

mod face;
mod freetype;

pub use face::Face;
pub use freetype::FreeType;

use core::result;

pub struct Error;

pub type Result<T> = result::Result<T, Error>;
