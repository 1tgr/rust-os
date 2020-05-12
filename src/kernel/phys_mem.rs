use crate::arch;
use crate::multiboot::{multiboot_info_t, multiboot_memory_map_t, multiboot_module_t, multiboot_uint32_t};
use crate::ptr::{self, Align, PointerInSlice};
use crate::spin::Mutex;
use bit_vec::BitVec;
use core::cmp;
use core::intrinsics;
use core::mem;
use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering};
use libc::{c_int, c_void};
use syscall::{ErrNum, Result};

extern "C" {
    static mut KERNEL_BASE: u8;
    static kernel_start: u8;
    static kernel_end: u8;
    static mut heap_start: u8;
    static heap_end: u8;
    static mboot_ptr: multiboot_uint32_t;
}

#[no_mangle]
pub unsafe extern "C" fn sbrk(incr: c_int) -> *mut c_void {
    static mut BRK: usize = 0;
    let begin = (&mut heap_start as *mut u8).offset(BRK as isize);
    assert!((begin as *const u8) < (&heap_end as *const u8), "out of heap space");
    BRK += incr as usize;
    begin as *mut c_void
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

    pub fn parse_multiboot() -> PhysicalBitmap {
        let info = multiboot_info();
        let kernel_len = ptr::bytes_between(unsafe { &kernel_start }, unsafe { &kernel_end });
        let lower_bytes = info.mem_lower as usize * 1024;
        let total_bytes = cmp::min(lower_bytes, 1024 * 1024) + (info.mem_upper as usize * 1024);
        let bitmap = PhysicalBitmap::new(total_bytes);
        bitmap.reserve_pages(0, 1);
        bitmap.reserve_ptr(unsafe { &kernel_start }, kernel_len as usize);
        bitmap.reserve_addr(lower_bytes, cmp::max(0, 1024 * 1024 - lower_bytes));
        bitmap.reserve_addr(unsafe { mboot_ptr } as usize, mem::size_of::<multiboot_info_t>());
        bitmap.reserve_addr(
            info.mods_addr as usize,
            info.mods_count as usize * mem::size_of::<multiboot_module_t>(),
        );

        {
            let mut mmap_offset = 0;
            while mmap_offset < info.mmap_length {
                let mmap: &multiboot_memory_map_t = unsafe { phys2virt((info.mmap_addr + mmap_offset) as usize) };
                if mmap._type != 1 {
                    bitmap.reserve_addr(mmap.addr as usize, mmap.len as usize);
                }

                mmap_offset += mmap.size + 4;
            }
        }

        let mods: &[multiboot_module_t] =
            unsafe { slice::from_raw_parts(phys2virt(info.mods_addr as usize), info.mods_count as usize) };

        for module in mods {
            let addr = module.mod_start;
            let len = module.mod_end - module.mod_start;
            bitmap.reserve_addr(addr as usize, len as usize);
        }

        bitmap
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

pub fn multiboot_info() -> &'static multiboot_info_t {
    unsafe { phys2virt(mboot_ptr as usize) }
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

        fn can_parse_multiboot() {
            let bitmap = PhysicalBitmap::parse_multiboot();
            let total_bytes = bitmap.total_bytes();
            let free_bytes = bitmap.free_bytes();
            assert!(total_bytes > 0);
            assert!(free_bytes > 0);
            assert!(free_bytes < total_bytes);
        }

        fn can_alloc_zeroed_memory() {
            let bitmap = PhysicalBitmap::parse_multiboot();
            let addr = bitmap.alloc_zeroed_page().unwrap();
            let ptr: &[u8; PAGE_SIZE] = unsafe { phys2virt(addr) };
            assert!(ptr.iter().all(|&b| b == 0));
        }
    }
}
