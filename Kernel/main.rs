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

#[macro_use]
extern crate core;
extern crate libc;
extern crate spin;

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

mod logging;
mod multiboot;
mod phys_mem;
mod prelude;
mod thread;
pub mod unwind;

use libc::{c_int,c_void};
use multiboot::{multiboot_info_t,multiboot_memory_map_t};
use phys_mem::PhysicalBitmap;
use std::cmp;
use thread::Scheduler;

extern {
    static kernel_start: i8;
    static mut kernel_end: i8;
    static mboot_ptr: multiboot::multiboot_uint32_t;
}

static mut brk: isize = 0;

#[no_mangle]
pub unsafe extern fn sbrk(incr: c_int) -> *mut c_void {
    let begin = (&mut kernel_end as *mut i8).offset(brk);
    brk += incr as isize;
    log!("sbrk({}) = {:x}", incr, begin as isize);
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

fn ptrdiff<T>(ptr1: *const T, ptr2: *const T) -> isize {
    ptr1 as isize - ptr2 as isize
}

// Kernel entrypoint
#[cfg(not(test))]
#[lang="start"]
#[no_mangle]
pub fn kmain() -> ! {
    log!("begin kmain");

    let bitmap = {
        let info: &multiboot::multiboot_info_t = phys_mem::phys2virt(mboot_ptr as usize);
        let total_kb = cmp::min(info.mem_lower, 1024) + info.mem_upper;
        let bitmap = PhysicalBitmap::new(total_kb as usize * 1024);

        {
            let kernel_end_ptr = unsafe { (&kernel_end as *const i8).offset(brk) };
            let kernel_len = ptrdiff(kernel_end_ptr, &kernel_start);
            bitmap.reserve_ptr(&kernel_start, kernel_len as usize);
        }

        {
            let mut mmap_offset = 0;
            while mmap_offset < info.mmap_length {
                let mmap: &multiboot::multiboot_memory_map_t = phys_mem::phys2virt((info.mmap_addr + mmap_offset) as usize);
                if mmap._type != 1 {
                    bitmap.reserve_addr(mmap.addr as usize, mmap.len as usize);
                }

                mmap_offset += mmap.size + 4;
            }
        }

        let bytes_free = bitmap.bytes_free();
        log!("free memory: {} bytes ({}KB)", bytes_free, bytes_free / 1024);
        bitmap
    };

    let addr = bitmap.alloc_page().unwrap();
    log!("alloc_page = {:x}", addr);
    bitmap.free_page(addr);

    let scheduler = Scheduler::new();
    let greeting = "hello";
    scheduler.spawn(move || log!("{} world", greeting));
    scheduler.spawn(move || log!("{} second thread", greeting));
    scheduler.schedule();
    log!("end kmain");
    loop {
        scheduler.schedule();
    }
}

