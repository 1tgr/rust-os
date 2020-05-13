use core::sync::atomic::{AtomicUsize, Ordering};

pub fn alloc_id() -> usize {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

#[cfg(target_os = "rust_os")]
mod rust_os;

#[cfg(not(target_os = "rust_os"))]
mod posix;

#[cfg(target_os = "rust_os")]
pub use rust_os::{AppSync, ClientPipe};

#[cfg(not(target_os = "rust_os"))]
pub use posix::{AppSync, ClientPipe};
