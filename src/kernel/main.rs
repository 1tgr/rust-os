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
#![feature(asm)]	//< As a kernel, we need inline assembly
#![feature(collections)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(heap_api)]
#![feature(lang_items)]	//< unwind needs to define lang items
#![feature(nonzero)]

#![no_std]

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use] extern crate collections;

#[macro_use] pub mod bitflags;
#[macro_use] pub mod lazy_static;
#[macro_use] pub mod mutex;
#[macro_use] pub mod macros;
#[macro_use] mod test;

extern crate alloc;
extern crate libc;
extern crate syscall;

// Achitecture-specific modules
#[cfg(target_arch="x86_64")]
#[path="arch/amd64/mod.rs"]
pub mod arch;
#[cfg(target_arch="x86")]
#[path="arch/x86/mod.rs"]
pub mod arch;

pub mod bit_vec;
pub mod console;
pub mod deferred;
pub mod elf;
pub mod io;
pub mod ksyscall;
pub mod libc_helpers;
pub mod logging;
pub mod multiboot;
pub mod once;
pub mod phys_mem;
pub mod prelude;
pub mod process;
pub mod ptr;
pub mod singleton;
pub mod tar;
pub mod thread;
pub mod unwind;
pub mod virt_mem;

#[cfg(feature = "test")]
mod demo;

#[cfg(feature = "test")]
fn run_tests() {
    use test::Fixture;

    const TEST_FIXTURES: &'static [Fixture] = &[
        ptr::test::TESTS,

        arch::isr::test::TESTS,
        arch::mmu::test::TESTS,
        io::pipe::test::TESTS,
        phys_mem::test::TESTS,
        virt_mem::test::TESTS,

        thread::test::TESTS,

        process::test::TESTS,
        // demo::TESTS
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
fn run_tests() {
}

// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub unsafe fn kmain() -> ! {
    arch::isr::init_once();
    libc_helpers::init();
    run_tests();
    loop {
        arch::cpu::wait_for_interrupt();
    }
}
