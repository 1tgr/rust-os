#![no_std]

#![feature(collections)]
#![feature(optin_builtin_traits)]
#![feature(unique)]

extern crate collections;
extern crate syscall;

mod file;
mod mutex;
mod oshandle;
mod osmem;
mod process;
mod sharedmem;

pub use self::file::*;
pub use self::mutex::*;
pub use self::oshandle::*;
pub use self::osmem::*;
pub use self::process::*;
pub use self::sharedmem::*;

pub type Result<T> = syscall::Result<T>;
