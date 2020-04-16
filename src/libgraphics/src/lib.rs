#[macro_use]
extern crate serde_derive;

extern crate alloc;

#[cfg(feature = "client")]
mod client;

#[cfg(feature = "server")]
mod server;

mod frame_buffer;
mod ipc;
mod types;

#[cfg(feature = "client")]
pub use client::{App, ClientPortal, Handler};

#[cfg(feature = "server")]
pub use server::{ServerApp, ServerInput, ServerPipe};

pub use types::{Event, EventInput, MouseButton, Rect};
