// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(alloc_error_handler)]
#![no_std]

extern crate libc;

// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
const MIN_ALIGN: size_t = 8;

use core::alloc::{GlobalAlloc, Layout};
use core::cmp;
use core::ptr;
use libc::{c_void, size_t};

extern "C" {
    fn memalign(align: size_t, size: size_t) -> *mut c_void;
}

struct LibcAllocator;

#[global_allocator]
static ALLOCATOR: LibcAllocator = LibcAllocator;

unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size() as size_t;
        let align = layout.align() as size_t;
        if align <= MIN_ALIGN {
            libc::malloc(size) as *mut u8
        } else {
            memalign(align, size) as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as *mut c_void)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let size = layout.size() as size_t;
        let align = layout.align() as size_t;
        let new_size = new_size as size_t;
        if align <= MIN_ALIGN {
            libc::realloc(ptr as *mut c_void, new_size) as *mut u8
        } else {
            let new_ptr = memalign(new_size, align) as *mut u8;
            ptr::copy(ptr, new_ptr, cmp::min(new_size, size) as usize);
            libc::free(ptr as *mut c_void);
            new_ptr
        }
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
