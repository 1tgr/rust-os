/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * main.rs
 * - Top-level file for kernel
 *
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */
#![feature(alloc)]
#![feature(asm)]	//< As a kernel, we need inline assembly
#![feature(collections)]
#![feature(core)]	//< libcore (see below) is not yet stablized
#![feature(lang_items)]	//< unwind needs to define lang items
#![feature(libc)]
#![feature(no_std)]	//< unwind needs to define lang items
#![cfg_attr(not(test), no_std)]	//< Kernels can't use std

use prelude::*;

// Load libcore (it's nice and freestanding)
// - We want the macros from libcore
#[macro_use]
extern crate core;

extern crate alloc;
extern crate collections;
extern crate libc;

/// A dummy 'std' module to work around a set of issues in rustc
#[cfg(not(test))]
mod std {
	// #18491 - write!() expands to std::fmt::Arguments::new
	pub use core::fmt;
	// #16803 - #[derive] references std::cmp
	pub use core::cmp;
	// #21827 - Slicing syntax references std::ops
	pub use core::ops;
	// #21827 - Loops reference std
	pub use core::iter;
	pub use core::option;
	// #16803 - Derive references marker/ops
	pub use core::marker;
    pub use alloc::boxed::Box;
    pub use collections::Vec;
}

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

// Achitecture-specific modules
#[cfg(all(not(test), target_arch="x86_64"))]
#[path="arch/amd64/mod.rs"]
pub mod arch;
#[cfg(all(not(test), target_arch="x86"))]
#[path="arch/x86/mod.rs"]
pub mod arch;
#[cfg(test)]
#[path="arch/test/mod.rs"]
pub mod arch;

// Prelude
mod prelude;

/// Exception handling (panic)
pub mod unwind;

/// Logging code
mod logging;

extern {
    static mut kernel_end: i8;
}

static mut brk: isize = 0;

#[no_mangle]
pub unsafe extern fn sbrk(incr: libc::c_int) -> *mut libc::c_void {
    let begin = (&mut kernel_end as *mut i8).offset(brk);
    brk = brk + incr as isize;
    begin as *mut libc::c_void
}

static mut errno: libc::c_int = 0;

#[no_mangle]
pub unsafe extern fn __error() -> &'static mut libc::c_int {
    &mut errno
}

#[test]
pub fn say_hello() {
    log!("hello world");
}

// Kernel entrypoint
#[cfg(not(test))]
#[lang="start"]
#[no_mangle]
pub fn kmain() {
    let mut stack = std::Vec::new();
    
    stack.push(1);
    stack.push(2);
    stack.push(3);
    
    while let Some(top) = stack.pop() {
        // Prints 3, 2, 1
        log!("{}", top);
    }
	loop {}
}

