#![crate_name = "graphics"]

#![feature(collections)]
#![feature(const_fn)]

#[macro_use] extern crate serde_derive;

extern crate cairo;
extern crate collections;
extern crate corepack;
extern crate os;
extern crate serde;
extern crate syscall;

mod client;
mod frame_buffer;
mod ipc;
mod types;
mod widget;

pub use client::*;
pub use frame_buffer::*;
pub use ipc::*;
pub use types::*;
pub use widget::*;
