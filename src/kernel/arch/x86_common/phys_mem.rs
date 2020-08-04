use crate::arch::multiboot::{multiboot_info_t, multiboot_memory_map_t, multiboot_module_t, multiboot_uint32_t};
use crate::phys_mem::{self, PhysicalBitmap};
use crate::ptr;
use core::{cmp, mem, slice};

extern "C" {
    static kernel_start: u8;
    static kernel_end: u8;
    static mboot_ptr: multiboot_uint32_t;
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
            let mmap: &multiboot_memory_map_t = unsafe { phys_mem::phys2virt((info.mmap_addr + mmap_offset) as usize) };
            if mmap._type != 1 {
                bitmap.reserve_addr(mmap.addr as usize, mmap.len as usize);
            }

            mmap_offset += mmap.size + 4;
        }
    }

    let mods: &[multiboot_module_t] =
        unsafe { slice::from_raw_parts(phys_mem::phys2virt(info.mods_addr as usize), info.mods_count as usize) };

    for module in mods {
        let addr = module.mod_start;
        let len = module.mod_end - module.mod_start;
        bitmap.reserve_addr(addr as usize, len as usize);
    }

    bitmap
}

pub fn multiboot_info() -> &'static multiboot_info_t {
    unsafe { phys_mem::phys2virt(mboot_ptr as usize) }
}

pub fn machine() -> PhysicalBitmap {
    parse_multiboot()
}

#[cfg(feature = "test")]
pub mod test {
    use super::*;
    use crate::phys_mem::{self, PAGE_SIZE};

    test! {
        fn can_parse_multiboot() {
            let bitmap = parse_multiboot();
            let total_bytes = bitmap.total_bytes();
            let free_bytes = bitmap.free_bytes();
            assert!(total_bytes > 0);
            assert!(free_bytes > 0);
            assert!(free_bytes < total_bytes);
        }

        fn can_alloc_zeroed_memory() {
            let bitmap = parse_multiboot();
            let addr = bitmap.alloc_zeroed_page().unwrap();
            let ptr: &[u8; PAGE_SIZE] = unsafe { phys_mem::phys2virt(addr) };
            assert!(ptr.iter().all(|&b| b == 0));
        }
    }
}
