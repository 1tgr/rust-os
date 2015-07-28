// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![unstable(feature = "heap_api",
            reason = "the precise API and guarantees it provides may be tweaked \
                      slightly, especially to possibly take into account the \
                      types being stored to make room for a future \
                      tracing garbage collector")]

use core::{isize, usize};

#[inline(always)]
fn check_size_and_alignment(size: usize, align: usize) {
    debug_assert!(size != 0);
    debug_assert!(size <= isize::MAX as usize, "Tried to allocate too much: {} bytes", size);
    debug_assert!(usize::is_power_of_two(align), "Invalid alignment of allocation: {}", align);
}

// FIXME: #13996: mark the `allocate` and `reallocate` return value as `noalias`

/// Return a pointer to `size` bytes of memory aligned to `align`.
///
/// On failure, return a null pointer.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
#[inline]
pub unsafe fn allocate(size: usize, align: usize) -> *mut u8 {
    check_size_and_alignment(size, align);
    imp::allocate(size, align)
}

/// Resize the allocation referenced by `ptr` to `size` bytes.
///
/// On failure, return a null pointer and leave the original allocation intact.
///
/// If the allocation was relocated, the memory at the passed-in pointer is
/// undefined after the call.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[inline]
pub unsafe fn reallocate(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8 {
    check_size_and_alignment(size, align);
    imp::reallocate(ptr, old_size, size, align)
}

/// Resize the allocation referenced by `ptr` to `size` bytes.
///
/// If the operation succeeds, it returns `usable_size(size, align)` and if it
/// fails (or is a no-op) it returns `usable_size(old_size, align)`.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[inline]
pub unsafe fn reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize,
                                 align: usize) -> usize {
    check_size_and_alignment(size, align);
    imp::reallocate_inplace(ptr, old_size, size, align)
}

/// Deallocates the memory referenced by `ptr`.
///
/// The `ptr` parameter must not be null.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[inline]
pub unsafe fn deallocate(ptr: *mut u8, old_size: usize, align: usize) {
    imp::deallocate(ptr, old_size, align)
}

/// Returns the usable size of an allocation created with the specified the
/// `size` and `align`.
#[inline]
pub fn usable_size(size: usize, align: usize) -> usize {
    imp::usable_size(size, align)
}

/// Prints implementation-defined allocator statistics.
///
/// These statistics may be inconsistent if other threads use the allocator
/// during the call.
pub fn stats_print() {
    imp::stats_print();
}

/// An arbitrary non-null address to represent zero-size allocations.
///
/// This preserves the non-null invariant for types like `Box<T>`. The address may overlap with
/// non-zero-size memory allocations.
pub const EMPTY: *mut () = 0x1 as *mut ();

/// The allocator for unique pointers.
#[cfg(not(test))]
#[lang = "exchange_malloc"]
#[inline]
unsafe fn exchange_malloc(size: usize, align: usize) -> *mut u8 {
    if size == 0 {
        EMPTY as *mut u8
    } else {
        let ptr = allocate(size, align);
        if ptr.is_null() { ::oom() }
        ptr
    }
}

#[cfg(not(test))]
#[lang = "exchange_free"]
#[inline]
unsafe fn exchange_free(ptr: *mut u8, old_size: usize, align: usize) {
    deallocate(ptr, old_size, align);
}

// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
#[cfg(all(not(feature = "external_funcs"),
          not(feature = "external_crate"),
          any(target_arch = "arm",
              target_arch = "mips",
              target_arch = "mipsel",
              target_arch = "powerpc")))]
const MIN_ALIGN: usize = 8;
#[cfg(all(not(feature = "external_funcs"),
          not(feature = "external_crate"),
          any(target_arch = "x86",
              target_arch = "x86_64",
              target_arch = "aarch64")))]
const MIN_ALIGN: usize = 16;

#[cfg(feature = "external_funcs")]
mod imp {
    #[allow(improper_ctypes)]
    extern {
        fn rust_allocate(size: usize, align: usize) -> *mut u8;
        fn rust_deallocate(ptr: *mut u8, old_size: usize, align: usize);
        fn rust_reallocate(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8;
        fn rust_reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize,
                                   align: usize) -> usize;
        fn rust_usable_size(size: usize, align: usize) -> usize;
        fn rust_stats_print();
    }

    #[inline]
    pub unsafe fn allocate(size: usize, align: usize) -> *mut u8 {
        rust_allocate(size, align)
    }

    #[inline]
    pub unsafe fn deallocate(ptr: *mut u8, old_size: usize, align: usize) {
        rust_deallocate(ptr, old_size, align)
    }

    #[inline]
    pub unsafe fn reallocate(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8 {
        rust_reallocate(ptr, old_size, size, align)
    }

    #[inline]
    pub unsafe fn reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize,
                                     align: usize) -> usize {
        rust_reallocate_inplace(ptr, old_size, size, align)
    }

    #[inline]
    pub fn usable_size(size: usize, align: usize) -> usize {
        unsafe { rust_usable_size(size, align) }
    }

    #[inline]
    pub fn stats_print() {
        unsafe { rust_stats_print() }
    }
}

#[cfg(feature = "external_crate")]
mod imp {
    extern crate external;
    pub use self::external::{allocate, deallocate, reallocate_inplace, reallocate};
    pub use self::external::{usable_size, stats_print};
}

#[cfg(all(not(feature = "external_funcs"),
          not(feature = "external_crate"),
          jemalloc))]
mod imp {
    use core::option::Option;
    use core::option::Option::None;
    use core::ptr::{null_mut, null};
    use libc::{c_char, c_int, c_void, size_t};
    use super::MIN_ALIGN;

    #[link(name = "jemalloc", kind = "static")]
    #[cfg(not(test))]
    extern {}

    extern {
        #[allocator]
        fn je_mallocx(size: size_t, flags: c_int) -> *mut c_void;
        fn je_rallocx(ptr: *mut c_void, size: size_t, flags: c_int) -> *mut c_void;
        fn je_xallocx(ptr: *mut c_void, size: size_t, extra: size_t, flags: c_int) -> size_t;
        fn je_sdallocx(ptr: *mut c_void, size: size_t, flags: c_int);
        fn je_nallocx(size: size_t, flags: c_int) -> size_t;
        fn je_malloc_stats_print(write_cb: Option<extern "C" fn(cbopaque: *mut c_void,
                                                                *const c_char)>,
                                 cbopaque: *mut c_void,
                                 opts: *const c_char);
    }

    // -lpthread needs to occur after -ljemalloc, the earlier argument isn't enough
    #[cfg(all(not(windows),
              not(target_os = "android"),
              not(target_env = "musl")))]
    #[link(name = "pthread")]
    extern {}

    // MALLOCX_ALIGN(a) macro
    #[inline(always)]
    fn mallocx_align(a: usize) -> c_int { a.trailing_zeros() as c_int }

    #[inline(always)]
    fn align_to_flags(align: usize) -> c_int {
        if align <= MIN_ALIGN { 0 } else { mallocx_align(align) }
    }

    #[inline]
    pub unsafe fn allocate(size: usize, align: usize) -> *mut u8 {
        let flags = align_to_flags(align);
        je_mallocx(size as size_t, flags) as *mut u8
    }

    #[inline]
    pub unsafe fn reallocate(ptr: *mut u8, _old_size: usize, size: usize, align: usize) -> *mut u8 {
        let flags = align_to_flags(align);
        je_rallocx(ptr as *mut c_void, size as size_t, flags) as *mut u8
    }

    #[inline]
    pub unsafe fn reallocate_inplace(ptr: *mut u8, _old_size: usize, size: usize,
                                     align: usize) -> usize {
        let flags = align_to_flags(align);
        je_xallocx(ptr as *mut c_void, size as size_t, 0, flags) as usize
    }

    #[inline]
    pub unsafe fn deallocate(ptr: *mut u8, old_size: usize, align: usize) {
        let flags = align_to_flags(align);
        je_sdallocx(ptr as *mut c_void, old_size as size_t, flags)
    }

    #[inline]
    pub fn usable_size(size: usize, align: usize) -> usize {
        let flags = align_to_flags(align);
        unsafe { je_nallocx(size as size_t, flags) as usize }
    }

    pub fn stats_print() {
        unsafe {
            je_malloc_stats_print(None, null_mut(), null())
        }
    }
}

#[cfg(all(not(feature = "external_funcs"),
          not(feature = "external_crate"),
          not(jemalloc),
          unix))]
mod imp {
    use core::cmp;
    use core::ptr;
    use libc;
    use super::MIN_ALIGN;

    extern {
        fn posix_memalign(memptr: *mut *mut libc::c_void,
                          align: libc::size_t,
                          size: libc::size_t) -> libc::c_int;
    }

    #[inline]
    pub unsafe fn allocate(size: usize, align: usize) -> *mut u8 {
        if align <= MIN_ALIGN {
            libc::malloc(size as libc::size_t) as *mut u8
        } else {
            let mut out = ptr::null_mut();
            let ret = posix_memalign(&mut out,
                                     align as libc::size_t,
                                     size as libc::size_t);
            if ret != 0 {
                ptr::null_mut()
            } else {
                out as *mut u8
            }
        }
    }

    #[inline]
    pub unsafe fn reallocate(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8 {
        if align <= MIN_ALIGN {
            libc::realloc(ptr as *mut libc::c_void, size as libc::size_t) as *mut u8
        } else {
            let new_ptr = allocate(size, align);
            ptr::copy(ptr, new_ptr, cmp::min(size, old_size));
            deallocate(ptr, old_size, align);
            new_ptr
        }
    }

    #[inline]
    pub unsafe fn reallocate_inplace(_ptr: *mut u8, old_size: usize, _size: usize,
                                     _align: usize) -> usize {
        old_size
    }

    #[inline]
    pub unsafe fn deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
        libc::free(ptr as *mut libc::c_void)
    }

    #[inline]
    pub fn usable_size(size: usize, _align: usize) -> usize {
        size
    }

    pub fn stats_print() {}
}

#[cfg(all(not(feature = "external_funcs"),
          not(feature = "external_crate"),
          not(jemalloc),
          windows))]
mod imp {
    use core::mem::size_of;
    use libc::{BOOL, DWORD, HANDLE, LPVOID, SIZE_T, INVALID_HANDLE_VALUE};
    use libc::{WriteFile};
    use super::MIN_ALIGN;

    extern "system" {
        fn GetProcessHeap() -> HANDLE;
        fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
        fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
        fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
        fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
        fn HeapSummary(hHeap: HANDLE, dwFlags: DWORD, lpSummary: LPHEAP_SUMMARY) -> BOOL;
    }

    #[repr(C)] #[allow(non_snake_case)]
    struct HEAP_SUMMARY {
        cb: DWORD,
        cbAllocated: SIZE_T,
        cbCommitted: SIZE_T,
        cbReserved: SIZE_T,
        cbMaxReserve: SIZE_T,
    }
    #[allow(non_camel_case_types)]
    type LPHEAP_SUMMARY = *mut HEAP_SUMMARY;

    #[repr(C)]
    struct Header(*mut u8);

    const HEAP_REALLOC_IN_PLACE_ONLY: DWORD = 0x00000010;
    const STD_OUTPUT_HANDLE: DWORD = -11i32 as u32;

    #[inline]
    unsafe fn get_header<'a>(ptr: *mut u8) -> &'a mut Header {
        &mut *(ptr as *mut Header).offset(-1)
    }

    #[inline]
    unsafe fn align_ptr(ptr: *mut u8, align: usize) -> *mut u8 {
        let aligned = ptr.offset((align - (ptr as usize & (align - 1))) as isize);
        *get_header(aligned) = Header(ptr);
        aligned
    }

    #[inline]
    pub unsafe fn allocate(size: usize, align: usize) -> *mut u8 {
        if align <= MIN_ALIGN {
            HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut u8
        } else {
            let ptr = HeapAlloc(GetProcessHeap(), 0, (size + align) as SIZE_T) as *mut u8;
            if ptr.is_null() { return ptr }
            align_ptr(ptr, align)
        }
    }

    #[inline]
    pub unsafe fn reallocate(ptr: *mut u8, _old_size: usize, size: usize, align: usize) -> *mut u8 {
        if align <= MIN_ALIGN {
            HeapReAlloc(GetProcessHeap(), 0, ptr as LPVOID, size as SIZE_T) as *mut u8
        } else {
            let header = get_header(ptr);
            let new = HeapReAlloc(GetProcessHeap(), 0, header.0 as LPVOID,
                                  (size + align) as SIZE_T) as *mut u8;
            if new.is_null() { return new }
            align_ptr(new, align)
        }
    }

    #[inline]
    pub unsafe fn reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize,
                                     align: usize) -> usize {
        if align <= MIN_ALIGN {
            let new = HeapReAlloc(GetProcessHeap(), HEAP_REALLOC_IN_PLACE_ONLY, ptr as LPVOID,
                                  size as SIZE_T) as *mut u8;
            if new.is_null() { old_size } else { size }
        } else {
            old_size
        }
    }

    #[inline]
    pub unsafe fn deallocate(ptr: *mut u8, _old_size: usize, align: usize) {
        if align <= MIN_ALIGN {
            let err = HeapFree(GetProcessHeap(), 0, ptr as LPVOID);
            debug_assert!(err != 0);
        } else {
            let header = get_header(ptr);
            let err = HeapFree(GetProcessHeap(), 0, header.0 as LPVOID);
            debug_assert!(err != 0);
        }
    }

    #[inline]
    pub fn usable_size(size: usize, _align: usize) -> usize {
        size
    }

    pub fn stats_print() {
        use core::fmt::{Error, Result, Write};
        use core::ptr::null_mut;
        use core::raw::Repr;
        use core::result::Result::{Ok, Err};
        struct Console(HANDLE);
        impl Write for Console {
            fn write_str(&mut self, s: &str) -> Result {
                let repr = s.repr();
                let mut written = 0;
                let err = unsafe { WriteFile(self.0, repr.data as LPVOID, repr.len as DWORD,
                                             &mut written, null_mut()) };
                if written as usize != repr.len { return Err(Error) }
                if err == 0 { return Err(Error) }
                Ok(())
            }
        }
        let mut hs = HEAP_SUMMARY {
            cb: size_of::<HEAP_SUMMARY>() as DWORD,
            cbAllocated: 0,
            cbCommitted: 0,
            cbReserved: 0,
            cbMaxReserve: 0,
        };
        let err = unsafe { HeapSummary(GetProcessHeap(), 0, &mut hs) };
        assert!(err != 0);
        let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        if handle.is_null() || handle == INVALID_HANDLE_VALUE { panic!("Failed to open stdout") }
        let mut out = Console(handle);
        writeln!(&mut out, "Allocated: {}", hs.cbAllocated).unwrap();
        writeln!(&mut out, "Committed: {}", hs.cbCommitted).unwrap();
        writeln!(&mut out, "Reserved: {}", hs.cbReserved).unwrap();
        writeln!(&mut out, "MaxReserve: {}", hs.cbMaxReserve).unwrap();
    }

    #[test]
    fn alignment_header_size() {
        assert!(size_of::<Header>() <= MIN_ALIGN);
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use boxed::Box;
    use heap;

    #[test]
    fn basic_reallocate_inplace_noop() {
        unsafe {
            let size = 4000;
            let ptr = heap::allocate(size, 8);
            if ptr.is_null() { ::oom() }
            let ret = heap::reallocate_inplace(ptr, size, size, 8);
            heap::deallocate(ptr, size, 8);
            assert_eq!(ret, heap::usable_size(size, 8));
        }
    }

    #[bench]
    fn alloc_owned_small(b: &mut Bencher) {
        b.iter(|| {
            let _: Box<_> = box 10;
        })
    }
}
