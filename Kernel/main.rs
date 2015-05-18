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
#![feature(asm)]	//< As a kernel, we need inline assembly
#![feature(alloc)]
#![feature(core)]
#![feature(lang_items)]	//< unwind needs to define lang items

use std::boxed::Box;
use libc::{c_int,c_void};

#[macro_use]
extern crate core;
extern crate libc;

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

mod prelude;
pub mod unwind;
mod logging;
mod thread;

extern {
    static mut kernel_end: i8;
}

static mut brk: isize = 0;

#[no_mangle]
pub unsafe extern fn sbrk(incr: c_int) -> *mut c_void {
    let begin = (&mut kernel_end as *mut i8).offset(brk);
    brk = brk + incr as isize;
    begin as *mut c_void
}

static mut errno: c_int = 0;

#[no_mangle]
pub unsafe extern fn __error() -> &'static mut c_int {
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
pub fn kmain() -> ! {
    log!("begin kmain");

    let who = "world abc";
    let say_hello = move || log!("hello {}", who);
    let t = thread::Thread::new(Box::new(say_hello));
    t.jump();
}

