#![feature(collections)]
#![feature(const_fn)]

#[macro_use] extern crate serde_derive;

extern crate cairo;
extern crate collections;
extern crate corepack;
extern crate os;
extern crate serde;
extern crate syscall;

#[cfg(feature = "test")]
#[macro_use]
extern crate testlite;

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

#[cfg(feature = "test")]
pub const TEST_FIXTURES: &'static [testlite::Fixture] = &[
    client::test::TESTS,
    frame_buffer::test::TESTS,
    ipc::test::TESTS,
    widget::test::TESTS,
];
