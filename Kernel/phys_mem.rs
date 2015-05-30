use libc::{c_int,c_void};
use spin::RwLock;
use std::cmp;
use std::collections::bit_vec::BitVec;
use std::iter::Iterator;
use std::option::Option::{Some,None};
use std::result::Result::{self,Ok,Err};
use super::multiboot::{multiboot_info_t,multiboot_memory_map_t,multiboot_uint32_t};

extern {
    static KERNEL_BASE: u8;
    static kernel_start: u8;
    static mut kernel_end: u8;
    static mboot_ptr: multiboot_uint32_t;
}

static mut brk: isize = 0;

#[no_mangle]
pub unsafe extern fn sbrk(incr: c_int) -> *mut c_void {
    let begin = (&mut kernel_end as *mut u8).offset(brk);
    brk += incr as isize;
    log!("sbrk({}) = {:x}", incr, begin as isize);
    begin as *mut c_void
}

pub const PAGE_SIZE: usize = 4096;

fn ptrdiff<T>(ptr1: *const T, ptr2: *const T) -> isize {
    ptr1 as isize - ptr2 as isize
}

pub struct PhysicalBitmap {
    free: RwLock<BitVec>
}

impl PhysicalBitmap {
    pub fn new(total_bytes: usize) -> PhysicalBitmap {
        let free = BitVec::from_elem(total_bytes / PAGE_SIZE, true);
        log!("total memory: {} bytes ({}KB)", free.len() * PAGE_SIZE, (free.len() * PAGE_SIZE) / 1024);
        PhysicalBitmap { free: RwLock::new(free) }
    }

    pub fn parse_multiboot(info: &multiboot_info_t, kernel_start_ptr: *const u8, kernel_len: usize) -> PhysicalBitmap {
        let total_kb = cmp::min(info.mem_lower, 1024) + info.mem_upper;
        let bitmap = PhysicalBitmap::new(total_kb as usize * 1024);
        bitmap.reserve_ptr(kernel_start_ptr, kernel_len as usize);

        {
            let mut mmap_offset = 0;
            while mmap_offset < info.mmap_length {
                let mmap: &multiboot_memory_map_t = phys2virt((info.mmap_addr + mmap_offset) as usize);
                if mmap._type != 1 {
                    bitmap.reserve_addr(mmap.addr as usize, mmap.len as usize);
                }

                mmap_offset += mmap.size + 4;
            }
        }

        let bytes_free = bitmap.bytes_free();
        log!("free memory: {} bytes ({}KB)", bytes_free, bytes_free / 1024);
        bitmap
    }

    pub fn reserve_pages(&self, start_page: usize, page_count: usize) {
        let mut free = self.free.write();
        log!("reserved {} bytes ({}KB) at {:x}", page_count * PAGE_SIZE, (page_count * PAGE_SIZE) / 1024, start_page * PAGE_SIZE);
        for i in 0..page_count - 1 {
            free.set(i, false);
        }
    }

    pub fn reserve_addr(&self, addr: usize, len: usize) {
        self.reserve_pages(addr / PAGE_SIZE, (len + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn reserve_ptr<T>(&self, ptr: *const T, len: usize) {
        let addr = virt2phys(ptr);
        self.reserve_addr(addr, len)
    }

    pub fn bytes_free(&self) -> usize {
        let free = self.free.read();
        let free_count = free.iter().filter(|x| *x).count();
        free_count * PAGE_SIZE
    }

    pub fn alloc_page(&self) -> Result<usize, &'static str> {
        let mut free = self.free.write();
        match free.iter().position(|x| x) {
            Some(i) => {
                free.set(i, false);
                Ok(i * PAGE_SIZE)
            }

            None => Err("out of memory")
        }
    }

    pub fn free_page(&self, addr: usize) {
        let mut free = self.free.write();
        let i = addr / PAGE_SIZE;
        free.set(i, true)
    }
}

pub fn phys2virt<T>(addr: usize) -> &'static T {
    let kernel_base_ptr: *const u8 = &KERNEL_BASE as *const u8;
    unsafe {
        let ptr: *const u8 = kernel_base_ptr.offset(addr as isize);
        let ptr: *const T = ptr as *const T;
        &*ptr
    }
}

pub fn virt2phys<T>(ptr: *const T) -> usize {
    let kernel_base_ptr: *const u8 = &KERNEL_BASE as *const u8;
    ptr as usize - kernel_base_ptr as usize
}

test!(
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
        assert_eq!(err, "out of memory");
    }

    fn can_parse_multiboot() {
        let info: &multiboot_info_t = phys2virt(mboot_ptr as usize);
        let kernel_len = unsafe { ptrdiff(&kernel_end, &kernel_start) + brk };
        PhysicalBitmap::parse_multiboot(&info, &kernel_start as *const u8, kernel_len as usize);
    }
);
