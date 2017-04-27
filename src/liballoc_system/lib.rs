// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(stage0, feature(custom_attribute))]
#![crate_name = "alloc_system"]
#![crate_type = "rlib"]
#![no_std]
#![cfg_attr(not(stage0), allocator)]
#![unstable(feature = "alloc_system",
            reason = "this library is unlikely to be stabilized in its current \
                      form or name",
            issue = "27783")]
#![feature(allocator)]
#![feature(staged_api)]

extern crate libc;

// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
const MIN_ALIGN: usize = 8;

#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe { imp::allocate(size, align) }
}

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize) {
    unsafe { imp::deallocate(ptr, old_size, align) }
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize,
                                align: usize) -> *mut u8 {
    unsafe { imp::reallocate(ptr, old_size, size, align) }
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(ptr: *mut u8, old_size: usize,
                                        size: usize, align: usize) -> usize {
    unsafe { imp::reallocate_inplace(ptr, old_size, size, align) }
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, align: usize) -> usize {
    imp::usable_size(size, align)
}

mod imp {
    use core::cmp;
    use core::ptr;
    use libc;
    use MIN_ALIGN;

    extern {
        fn memalign(align: libc::size_t, size: libc::size_t) -> *mut libc::c_void;
    }

    pub unsafe fn allocate(size: usize, align: usize) -> *mut u8 {
        if align <= MIN_ALIGN {
            libc::malloc(size as libc::size_t) as *mut u8
        } else {
            memalign(align as libc::size_t, size as libc::size_t) as *mut u8
        }
    }

    pub unsafe fn reallocate(ptr: *mut u8, old_size: usize, size: usize,
                             align: usize) -> *mut u8 {
        if align <= MIN_ALIGN {
            libc::realloc(ptr as *mut libc::c_void, size as libc::size_t) as *mut u8
        } else {
            let new_ptr = allocate(size, align);
            ptr::copy(ptr, new_ptr, cmp::min(size, old_size));
            deallocate(ptr, old_size, align);
            new_ptr
        }
    }

    pub unsafe fn reallocate_inplace(_ptr: *mut u8, old_size: usize, _size: usize,
                                     _align: usize) -> usize {
        old_size
    }

    pub unsafe fn deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
        libc::free(ptr as *mut libc::c_void)
    }

    pub fn usable_size(size: usize, _align: usize) -> usize {
        size
    }
}
