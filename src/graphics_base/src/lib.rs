extern crate alloc;

#[macro_use]
extern crate serde_derive;

#[cfg(target_os = "rust_os")]
mod compat {
    pub use syscall::{ErrNum as Error, Result};
}

#[cfg(not(target_os = "rust_os"))]
mod compat {
    use std::result;

    #[derive(Clone, Debug)]
    pub enum Error {
        NotSupported,
    }

    pub type Result<T> = result::Result<T, Error>;
}

#[cfg(target_os = "rust_os")]
pub mod ipc;

pub mod frame_buffer;
pub mod system;
pub mod types;

pub use compat::{Error, Result};
