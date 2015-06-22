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
#![feature(core)]
#![feature(lang_items)]	//< unwind needs to define lang items

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use] extern crate bitflags;
#[macro_use] extern crate core;
#[macro_use] extern crate lazy_static;

#[macro_use] mod macros;
#[macro_use] mod test;

extern crate alloc;
extern crate libc;
extern crate spin;

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

mod logging;
mod multiboot;
mod prelude;
mod process;
mod thread;
mod virt_mem;
pub mod phys_mem;
pub mod unwind;

use core::fmt::Write;
use libc::{c_char,c_int};
use logging::Writer;
use std::mem;
use test::Fixture;

static mut errno: c_int = 0;

#[no_mangle]
pub unsafe extern fn __error() -> &'static mut c_int {
    &mut errno
}

#[no_mangle]
pub unsafe extern fn __assert(file: *const c_char, line: c_int, msg: *const c_char) -> ! {
    let mut writer = Writer::get(module_path!());
    arch::debug::put_cstr(file);
    let _ = write!(&mut writer, "({}): ", line);
    arch::debug::put_cstr(msg);
    mem::drop(writer);
    panic!("assertion failed in C code");
}

const TEST_FIXTURES: &'static [Fixture] = &[
    arch::isr::TESTS,
    arch::process::TESTS,
    phys_mem::TESTS,
    process::TESTS,
    thread::TESTS,
    virt_mem::TESTS,
];

// Kernel entrypoint
#[cfg(not(test))]
#[lang="start"]
#[no_mangle]
pub fn kmain() -> ! {
    log!("begin kmain");

    for &(fixture_name, fixture) in TEST_FIXTURES {
        for &(test_name, test_fn) in fixture {
            log!("begin {}::{}", fixture_name, test_name);
            test_fn();
            log!("end {}::{}\n", fixture_name, test_name);
        }
    }

    log!("end kmain");
    loop { }
}

