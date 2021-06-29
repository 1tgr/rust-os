#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod copy_strides;
mod fill_strides;
mod sprite;

pub use crate::sprite::Sprite;
