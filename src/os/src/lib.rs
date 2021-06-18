#![no_std]
#![feature(lang_items)]
#![feature(thread_local)]

extern crate alloc;
extern crate syscall;

pub mod libc_helpers;

mod detail;
mod file;
mod mutex;
mod oshandle;
mod osmem;
mod process;
mod semaphore;
mod sharedmem;
mod thread;

pub use self::file::*;
pub use self::mutex::*;
pub use self::oshandle::*;
pub use self::osmem::*;
pub use self::process::*;
pub use self::semaphore::*;
pub use self::sharedmem::*;
pub use self::thread::*;

pub type Result<T> = syscall::Result<T>;
