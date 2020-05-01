#[macro_use]
extern crate serde_derive;

extern crate alloc;

#[cfg(target_os = "rust_os")]
mod compat {
    pub use os::Mutex;
    pub use syscall::{ErrNum as Error, Result};
}

#[cfg(not(target_os = "rust_os"))]
mod compat {
    use std::result;

    pub use std::sync::Mutex;

    #[derive(Clone, Debug)]
    pub enum Error {
        NotSupported,
    }

    pub type Result<T> = result::Result<T, Error>;
}

#[cfg(feature = "client")]
mod client;

#[cfg(target_os = "rust_os")]
mod ipc;

#[cfg(not(target_os = "rust_os"))]
mod frame_buffer;

#[cfg(not(target_os = "rust_os"))]
mod server;

mod system;
mod types;

pub mod components;

#[cfg(all(target_os = "rust_os", feature = "server"))]
pub mod frame_buffer;

#[cfg(all(target_os = "rust_os", feature = "server"))]
pub mod server;

#[cfg(feature = "client")]
pub mod widgets;

#[cfg(feature = "client")]
pub use client::App;

pub use compat::{Error, Result};
pub use system::System;
pub use types::{Event, EventInput, MouseButton, MouseInput, Rect};
