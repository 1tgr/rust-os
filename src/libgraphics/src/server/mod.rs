#[cfg(target_os = "rust_os")]
mod app;

#[cfg(target_os = "rust_os")]
mod pipe;

pub(crate) mod portal;
pub(crate) mod screen;

#[cfg(target_os = "rust_os")]
pub use app::ServerApp;

#[cfg(target_os = "rust_os")]
pub use pipe::ServerPipe;

pub use portal::{PortalRef, ServerPortalSystem};
pub use screen::Screen;
