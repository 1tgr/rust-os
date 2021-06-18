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
#![feature(asm)] //< As a kernel, we need inline assembly
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(lang_items)] //< unwind needs to define lang items
#![feature(link_args)]
#![feature(panic_info_message)]
#![feature(start)]
#![no_std]
#![cfg_attr(target_arch = "arm", allow(dead_code))]
#![cfg_attr(target_arch = "arm", allow(unused_imports))]

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;
#[macro_use]
mod spin;
#[macro_use]
mod test;

extern crate alloc_system;

mod arch;
#[cfg(not(target_arch = "arm"))]
mod deferred;
mod elf;
#[cfg(not(target_arch = "arm"))]
mod io;
#[cfg(not(target_arch = "arm"))]
mod kobj;
#[cfg(not(target_arch = "arm"))]
mod ksyscall;
mod libc_helpers;
mod logging;
#[cfg(not(target_arch = "arm"))]
mod mutex;
mod once;
mod phys_mem;
mod prelude;
#[cfg(not(target_arch = "arm"))]
mod process;
mod ptr;
#[cfg(not(target_arch = "arm"))]
mod semaphore;
mod singleton;
mod tar;
#[cfg(not(target_arch = "arm"))]
mod thread;
mod unwind;
mod virt_mem;

#[cfg(feature = "test")]
#[cfg(not(target_arch = "arm"))]
mod demo;

#[cfg(feature = "test")]
fn run_tests() {
    use crate::test::Fixture;

    const TEST_FIXTURES: &'static [Fixture] = &[
        ptr::test::TESTS,
        #[cfg(not(target_arch = "arm"))]
        arch::isr::test::TESTS,
        #[cfg(not(target_arch = "arm"))]
        arch::mmu::test::TESTS,
        #[cfg(not(target_arch = "arm"))]
        arch::phys_mem::test::TESTS,
        #[cfg(not(target_arch = "arm"))]
        io::pipe::test::TESTS,
        phys_mem::test::TESTS,
        virt_mem::test::TESTS,
        #[cfg(not(target_arch = "arm"))]
        thread::test::TESTS,
        #[cfg(not(target_arch = "arm"))]
        mutex::test::TESTS,
        #[cfg(not(target_arch = "arm"))]
        process::test::TESTS,
        #[cfg(not(target_arch = "arm"))]
        semaphore::test::TESTS,
        #[cfg(not(target_arch = "arm"))]
        demo::TESTS,
    ];

    log!("begin kmain");

    for &(fixture_name, fixture) in TEST_FIXTURES {
        for &(test_name, test_fn) in fixture {
            log!("begin {}::{}", fixture_name, test_name);
            test_fn();
            log!("end {}::{}\n", fixture_name, test_name);
        }
    }

    log!("end kmain");
}

#[cfg(not(feature = "test"))]
fn run_tests() {}

// Kernel entrypoint
#[no_mangle]
pub unsafe fn kmain() -> ! {
    #[cfg(not(target_arch = "arm"))]
    arch::isr::init_once();
    libc_helpers::init();
    run_tests();
    loop {
        arch::cpu::wait_for_interrupt();
    }
}

#[lang = "start"]
#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    0
}
