use bit_vec::BitVec;
use core::cmp;
use core::intrinsics;
use libc::{c_int,c_void};
use multiboot::{multiboot_info_t,multiboot_memory_map_t,multiboot_uint32_t};
use mutex::Mutex;
use ptr;
use syscall::{ErrNum,Result};

extern {
    static mut KERNEL_BASE: u8;
    static kernel_start: u8;
    static kernel_end: u8;
    static mut heap_start: u8;
    static heap_end: u8;
    static mboot_ptr: multiboot_uint32_t;
}

#[no_mangle]
pub unsafe extern fn sbrk(incr: c_int) -> *mut c_void {
    static mut BRK: usize = 0;
    let begin = (&mut heap_start as *mut u8).offset(BRK as isize);
    assert!((begin as *const u8) < (&heap_end as *const u8), "out of heap space");
    BRK += incr as usize;
    begin as *mut c_void
}

pub const PAGE_SIZE: usize = 4096;

pub struct PhysicalBitmap {
    free: Mutex<BitVec>
}

impl PhysicalBitmap {
    pub fn new(total_bytes: usize) -> PhysicalBitmap {
        let free = BitVec::from_elem(total_bytes / PAGE_SIZE, true);
        PhysicalBitmap { free: Mutex::new(free) }
    }

    pub fn parse_multiboot() -> PhysicalBitmap {
        let info = multiboot_info();
        let kernel_len = ptr::bytes_between(&kernel_start, &kernel_end);
        let total_kb = cmp::min(info.mem_lower, 1024) + info.mem_upper;
        let bitmap = PhysicalBitmap::new(total_kb as usize * 1024);
        bitmap.reserve_ptr(&kernel_start, kernel_len as usize);

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
        let addr = {
            let mut free = lock!(self.free);
            match free.iter().position(|x| x) {
                Some(i) => {
                    free.set(i, false);
                    i * PAGE_SIZE
                }

                None => { return Err(ErrNum::OutOfMemory); }
            }
        };

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

pub unsafe fn phys2virt<T>(addr: usize) -> &'static mut T {
    let kernel_base_ptr: *mut u8 = &mut KERNEL_BASE as *mut u8;
    let ptr: *mut u8 = kernel_base_ptr.offset(addr as isize);
    let ptr: *mut T = ptr as *mut T;
    &mut *ptr
}

pub fn virt2phys<T>(ptr: *const T) -> usize {
    let kernel_base_ptr: *const u8 = unsafe { &KERNEL_BASE as *const u8 };
    ptr as usize - kernel_base_ptr as usize
}

pub fn multiboot_info() -> &'static multiboot_info_t {
    unsafe { phys2virt(mboot_ptr as usize) }
}

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

    fn alloc_returns_zeroed_memory() {
        let bitmap = PhysicalBitmap::parse_multiboot();
        let addr = bitmap.alloc_page().unwrap();
        let ptr: &[u8; PAGE_SIZE] = unsafe { phys2virt(addr) };
        assert!(ptr.iter().all(|&b| b == 0));
    }
}
