#![crate_name = "graphics"]

#![feature(const_fn)]

#[macro_use] extern crate serde_derive;

extern crate cairo;
extern crate corepack;
extern crate os;
extern crate serde;
extern crate syscall;

mod client;
mod frame_buffer;
mod ipc;
mod types;

pub use client::*;
pub use frame_buffer::*;
pub use ipc::*;
pub use types::*;
