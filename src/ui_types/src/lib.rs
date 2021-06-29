extern crate alloc;

#[macro_use]
extern crate serde_derive;

pub mod ipc;
pub mod types;

pub use syscall::{ErrNum as Error, Result};
