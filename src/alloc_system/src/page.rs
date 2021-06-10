use core::alloc::{GlobalAlloc, Layout};

pub struct PageAllocator;

unsafe impl GlobalAlloc for PageAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        assert!(layout.align() <= 4096);
        syscall::alloc_pages(size).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        syscall::free_pages(ptr);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.alloc(layout)
    }
}
