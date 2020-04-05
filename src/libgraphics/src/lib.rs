#[macro_use]
extern crate serde_derive;

extern crate alloc;
extern crate cairo;
extern crate core;
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

pub use crate::client::*;
pub use crate::frame_buffer::*;
pub use crate::ipc::*;
pub use crate::types::*;
pub use crate::widget::*;

#[cfg(feature = "test")]
pub const TEST_FIXTURES: &'static [testlite::Fixture] = &[
    client::test::TESTS,
    frame_buffer::test::TESTS,
    ipc::test::TESTS,
    widget::test::TESTS,
];
