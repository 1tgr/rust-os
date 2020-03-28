#![crate_name = "rt"]
#![feature(lang_items)]
#![feature(link_args)]
#![feature(panic_info_message)]
#![feature(panic_runtime)]
#![feature(rustc_private)]
#![feature(start)]
#![panic_runtime]
#![no_std]

extern crate libc;
extern crate syscall;

mod start;
mod unwind;
