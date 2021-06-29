use core::alloc::{GlobalAlloc, Layout};
use core::slice;

pub struct CheckAllocator<T>(pub T);

unsafe impl<T> GlobalAlloc for CheckAllocator<T>
where
    T: GlobalAlloc,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let layout_extra = Layout::from_size_align_unchecked(layout.size() + 128, layout.align());
        let ptr = self.0.alloc(layout_extra);
        if !ptr.is_null() {
            let s = slice::from_raw_parts_mut(ptr, layout_extra.size());
            let (_, right) = s.split_at_mut(layout.size());
            for dest in right {
                *dest = 0xdd;
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let layout_extra = Layout::from_size_align_unchecked(layout.size() + 128, layout.align());
        let s = slice::from_raw_parts(ptr, layout_extra.size());
        let (left, right) = s.split_at(layout.size());

        for (offset, &dest) in right.iter().enumerate() {
            assert_eq!(dest, 0xdd, "at {:p} + {} + {}", ptr, left.len(), offset);
        }

        self.0.dealloc(ptr, layout_extra);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let layout_extra = Layout::from_size_align_unchecked(layout.size() + 128, layout.align());
        let ptr = self.0.alloc_zeroed(layout_extra);
        if !ptr.is_null() {
            let s = slice::from_raw_parts_mut(ptr, layout_extra.size());
            let (_, right) = s.split_at_mut(layout.size());
            for dest in right {
                *dest = 0xdd;
            }
        }

        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let layout_extra = Layout::from_size_align_unchecked(layout.size() + 128, layout.align());
        let new_size_extra = new_size + 128;
        let s = slice::from_raw_parts(ptr, layout_extra.size());
        let (left, right) = s.split_at(layout.size());

        for (offset, &dest) in right.iter().enumerate() {
            assert_eq!(dest, 0xdd, "at {:p} + {} + {}", ptr, left.len(), offset);
        }

        let ptr = self.0.realloc(ptr, layout_extra, new_size_extra);

        if !ptr.is_null() {
            let s = slice::from_raw_parts_mut(ptr, new_size_extra);
            let (_, right) = s.split_at_mut(new_size);
            for dest in right {
                *dest = 0xdd;
            }
        }

        ptr
    }
}
