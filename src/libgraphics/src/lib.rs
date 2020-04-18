#![cfg_attr(all(not(test), not(target_os = "rust_os")), allow(dead_code))]

#[macro_use]
extern crate serde_derive;

extern crate alloc;

#[cfg(target_os = "rust_os")]
mod compat {
    pub use syscall::{ErrNum as Error, Result};

    pub(crate) use cairo::cairo::Cairo;
}

#[cfg(not(target_os = "rust_os"))]
mod compat {
    use core::result;

    pub enum Error {
        NotSupported,
    }

    pub type Result<T> = result::Result<T, Error>;

    pub struct Cairo;
}

#[cfg(all(target_os = "rust_os", feature = "client"))]
mod client;

#[cfg(all(target_os = "rust_os", feature = "server"))]
mod server;

#[cfg(target_os = "rust_os")]
mod frame_buffer;

#[cfg(target_os = "rust_os")]
mod ipc;

mod components;
mod system;
mod types;

#[cfg(all(target_os = "rust_os", feature = "client"))]
pub use client::{App, ClientPortal};

#[cfg(all(target_os = "rust_os", feature = "server"))]
pub use server::{ServerApp, ServerInput, ServerPipe};

pub use compat::{Error, Result};
pub use components::{NeedsPaint, OnInput, OnPaint, Position};
pub use system::System;
pub use types::{Event, EventInput, MouseButton, Rect};
