use crate::arch;
use crate::arch::phys_mem;
use crate::ptr::{self, Align, PointerInSlice};
use crate::spin::Mutex;
use bit_vec::BitVec;
use core::cmp;
use core::intrinsics;
use core::mem;
use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering};
use libc::c_void;
use syscall::{ErrNum, Result};

extern "C" {
    static mut KERNEL_BASE: u8;
    static kernel_end: u8;
    static mut heap_start: u8;
    static heap_end: u8;
}

pub unsafe fn resize_kernel_heap(delta: isize) -> *mut u8 {
    static mut BRK: usize = 0;
    let begin = (&mut heap_start as *mut u8).offset(BRK as isize);
    assert!((begin as *const u8) < (&heap_end as *const u8), "out of heap space");
    BRK += delta as usize;
    begin
}

static MALLOC_LOCK: AtomicUsize = AtomicUsize::new(0);
static mut MALLOC_LOCK_TOKEN: usize = 0;

#[no_mangle]
pub extern "C" fn __malloc_lock(_reent: *mut c_void) {
    let token = arch::disable_interrupts();
    // TODO: multi CPU
    if MALLOC_LOCK.fetch_add(1, Ordering::SeqCst) == 0 {
        unsafe {
            MALLOC_LOCK_TOKEN = token;
        }
    }
}

#[no_mangle]
pub extern "C" fn __malloc_unlock(_reent: *mut c_void) {
    // TODO: multi CPU
    if MALLOC_LOCK.fetch_sub(1, Ordering::SeqCst) == 1 {
        let token = mem::replace(unsafe { &mut MALLOC_LOCK_TOKEN }, 0);
        arch::restore_interrupts(token);
    }
}

pub const PAGE_SIZE: usize = 4096;

pub struct PhysicalBitmap {
    free: Mutex<BitVec>,
}

impl PhysicalBitmap {
    pub fn new(total_bytes: usize) -> PhysicalBitmap {
        let free = BitVec::from_elem(total_bytes / PAGE_SIZE, true);
        PhysicalBitmap { free: Mutex::new(free) }
    }

    pub fn machine() -> PhysicalBitmap {
        phys_mem::machine()
    }

    pub fn reserve_pages(&self, start_page: usize, page_count: usize) {
        let mut free = lock!(self.free);
        if start_page <= free.len() {
            let page_count = cmp::min(page_count, free.len() - start_page);
            for i in start_page..start_page + page_count {
                free.set(i, false);
            }
        }
    }

    pub fn reserve_addr(&self, addr: usize, len: usize) {
        self.reserve_pages(addr / PAGE_SIZE, (len + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn reserve_ptr<T>(&self, ptr: *const T, len: usize) {
        let addr = virt2phys(ptr);
        self.reserve_addr(addr, len)
    }

    pub fn total_bytes(&self) -> usize {
        let total_count = lock!(self.free).len();
        total_count * PAGE_SIZE
    }

    pub fn free_bytes(&self) -> usize {
        let free = lock!(self.free);
        let free_count = free.iter().filter(|x| *x).count();
        free_count * PAGE_SIZE
    }

    pub fn alloc_page(&self) -> Result<usize> {
        let mut free = lock!(self.free);
        match free.iter().position(|x| x) {
            Some(i) => {
                free.set(i, false);
                Ok(i * PAGE_SIZE)
            }

            None => Err(ErrNum::OutOfMemory),
        }
    }

    pub fn alloc_zeroed_page(&self) -> Result<usize> {
        let addr = self.alloc_page()?;

        unsafe {
            let ptr: &mut u8 = phys2virt(addr);
            intrinsics::write_bytes(ptr, 0, PAGE_SIZE);
        }

        Ok(addr)
    }

    pub fn free_page(&self, addr: usize) {
        let mut free = lock!(self.free);
        let i = addr / PAGE_SIZE;
        free.set(i, true)
    }
}

pub fn identity_range() -> &'static [u8] {
    let gigabyte = 1024 * 1024 * 1024;
    unsafe {
        let base_ptr = &KERNEL_BASE as *const u8;
        let end_ptr = Align::up(&kernel_end as *const u8, gigabyte);
        let len = ptr::bytes_between(base_ptr, end_ptr);
        slice::from_raw_parts(base_ptr, len)
    }
}

fn check_identity(addr: usize, ptr: *const u8) {
    let identity = identity_range();
    if !identity.contains_ptr(ptr) {
        panic!(
            "physical {:x}/virtual {:p} can't be contained in the identity mapping {:p}..{:p}",
            addr,
            ptr,
            identity.as_ptr(),
            unsafe { identity.as_ptr().offset(identity.len() as isize) }
        );
    }
}

pub unsafe fn phys2virt<T>(addr: usize) -> &'static mut T {
    let kernel_base_ptr: *mut u8 = &mut KERNEL_BASE as *mut u8;
    let ptr: *mut u8 = kernel_base_ptr.offset(addr as isize);
    check_identity(addr, ptr);

    let ptr: *mut T = ptr as *mut T;
    &mut *ptr
}

pub fn virt2phys<T>(ptr: *const T) -> usize {
    let kernel_base_ptr: *const u8 = unsafe { &KERNEL_BASE as *const u8 };
    let addr = ptr as usize - kernel_base_ptr as usize;
    check_identity(addr, ptr as *const u8);
    addr
}

#[cfg(feature = "test")]
pub mod test {
    use super::*;
    use syscall::ErrNum;

    test! {
        fn can_alloc_two_pages() {
            let bitmap = PhysicalBitmap::new(640 * 1024);
            let addr1 = bitmap.alloc_page().unwrap();
            let addr2 = bitmap.alloc_page().unwrap();
            assert!(addr1 != addr2);
        }

        fn can_alloc_free_realloc() {
            let bitmap = PhysicalBitmap::new(640 * 1024);
            let addr1 = bitmap.alloc_page().unwrap();
            bitmap.free_page(addr1);

            let addr2 = bitmap.alloc_page().unwrap();
            assert_eq!(addr1, addr2);
        }

        fn can_handle_out_of_memory() {
            let bitmap = PhysicalBitmap::new(2 * PAGE_SIZE);
            bitmap.alloc_page().unwrap();
            bitmap.alloc_page().unwrap();

            let err = bitmap.alloc_page().unwrap_err();
            assert_eq!(err, ErrNum::OutOfMemory);
        }
    }
}
