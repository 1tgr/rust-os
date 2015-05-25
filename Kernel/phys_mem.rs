use spin::RwLock;
use std::collections::bit_vec::BitVec;
use std::iter::Iterator;
use std::option::Option::{Some,None};
use std::result::Result::{self,Ok,Err};

extern {
    static KERNEL_BASE: i8;
}

pub const PAGE_SIZE: usize = 4096;

pub struct PhysicalBitmap {
    free: RwLock<BitVec>
}

impl PhysicalBitmap {
    pub fn new(total_bytes: usize) -> PhysicalBitmap {
        let free = BitVec::from_elem(total_bytes / PAGE_SIZE, true);
        log!("total memory: {} bytes ({}KB)", free.len() * PAGE_SIZE, (free.len() * PAGE_SIZE) / 1024);
        PhysicalBitmap { free: RwLock::new(free) }
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
    let kernel_base_ptr: *const i8 = &KERNEL_BASE as *const i8;
    unsafe {
        let ptr: *const i8 = kernel_base_ptr.offset(addr as isize);
        let ptr: *const T = ptr as *const T;
        &*ptr
    }
}

pub fn virt2phys<T>(ptr: *const T) -> usize {
    let kernel_base_ptr: *const i8 = &KERNEL_BASE as *const i8;
    ptr as usize - kernel_base_ptr as usize
}

test!(
    fn can_alloc_free() {
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
);
