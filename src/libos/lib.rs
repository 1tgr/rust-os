#![no_std]

#![feature(alloc)]
#![feature(collections)]
#![feature(fnbox)]
#![feature(optin_builtin_traits)]
#![feature(unique)]

extern crate alloc;
extern crate collections;
extern crate syscall;

mod file;
mod mutex;
mod oshandle;
mod osmem;
mod process;
mod sharedmem;
mod thread;

pub use self::file::*;
pub use self::mutex::*;
pub use self::oshandle::*;
pub use self::osmem::*;
pub use self::process::*;
pub use self::sharedmem::*;
pub use self::thread::*;

pub type Result<T> = syscall::Result<T>;
