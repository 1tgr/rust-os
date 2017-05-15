#![no_std]

#![feature(collections)]
#![feature(unique)]

extern crate collections;
extern crate syscall;

mod file;
mod oshandle;
mod osmem;
mod process;
mod sharedmem;

pub use self::file::*;
pub use self::oshandle::*;
pub use self::osmem::*;
pub use self::process::*;
pub use self::sharedmem::*;

pub type Result<T> = syscall::Result<T>;
