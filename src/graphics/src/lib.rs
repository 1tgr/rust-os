#![cfg_attr(target_os = "rust_os", no_std)]

extern crate alloc;

mod app;
mod pipe;
mod portal;

pub mod components;
pub mod widgets;

pub use app::App;
pub use graphics_base::types::*;
pub use graphics_base::Result;
pub use pipe::AppSync;
