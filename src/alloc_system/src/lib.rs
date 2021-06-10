#![feature(alloc_error_handler)]
#![no_std]

use core::alloc::Layout;

#[cfg(feature = "page_alloc")]
#[global_allocator]
static ALLOCATOR: check::CheckAllocator<page::PageAllocator> = check::CheckAllocator(page::PageAllocator);

#[cfg(not(feature = "page_alloc"))]
#[global_allocator]
static ALLOCATOR: libc::LibcAllocator = libc::LibcAllocator;

mod check;
mod libc;

#[cfg(feature = "page_alloc")]
mod page;

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
