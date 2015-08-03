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
#![crate_name = "kernel"]

#![feature(alloc)]
#![feature(append)]
#![feature(asm)]	//< As a kernel, we need inline assembly
#![feature(box_raw)]
#![feature(core)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(heap_api)]
#![feature(lang_items)]	//< unwind needs to define lang items
#![feature(slice_bytes)]

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use] extern crate bitflags;
#[macro_use] extern crate core;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate spin;

#[macro_use] mod macros;
#[macro_use] mod test;

extern crate alloc;
extern crate bit_vec;
extern crate libc;
extern crate miniz_sys;
extern crate syscall;

// Achitecture-specific modules
#[cfg(target_arch="x86_64")]
#[path="arch/amd64/mod.rs"]
pub mod arch;
#[cfg(target_arch="x86")]
#[path="arch/x86/mod.rs"]
pub mod arch;

pub mod device;
pub mod logging;
pub mod multiboot;
pub mod phys_mem;
pub mod process;
pub mod ptr;
pub mod singleton;
pub mod thread;
pub mod unwind;
pub mod virt_mem;

mod demo;

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
    ptr::TESTS,

    arch::isr::TESTS,
    arch::keyboard::TESTS,
    arch::mmu::TESTS,
    phys_mem::TESTS,
    process::TESTS,
    thread::TESTS,
    virt_mem::TESTS,

    demo::TESTS
];

// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kmain() -> ! {
    arch::isr::init_once();

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

