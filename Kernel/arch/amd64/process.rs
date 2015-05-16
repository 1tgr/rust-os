use ::phys_mem::{self,PhysicalBitmap};
use ::thread;
use std::marker::PhantomData;
use std::ops::Fn;
use std::option::Option::*;

pub struct ArchProcess {
    dummy: i32
}

enum PageFlags {
    Present = 0x001,
    Writable = 0x002,
    User = 0x004,
    Writethrough = 0x008,
    NoCache = 0x010,
    Accessed = 0x020,
    Dirty = 0x040, // PTE only
    FourMeg = 0x080, // PDE only
    Global = 0x100, // PTE only
    All = 0x1ff
}

#[repr(C)]
struct PageEntry<T> {
    entry: usize,
    phantom: PhantomData<T>
}

impl<T> PageEntry<T> {
    pub fn ensure_present(&mut self, alloc: &Fn() -> usize) -> bool {
        let flags = self.entry & (PageFlags::All as usize);
        if (flags & (PageFlags::Present as usize)) == 0 {
            self.entry = alloc() | flags | (PageFlags::Present as usize) | (PageFlags::Writable as usize) | (PageFlags::User as usize);
            true
        } else {
            false
        }
    }

    pub fn map(&mut self, addr: usize, user: bool, writable: bool) {
        let mut entry = addr | (PageFlags::Present as usize);
        if user {
            entry |= PageFlags::User as usize;
        }

        if writable {
            entry |= PageFlags::Writable as usize;
        }

        self.entry = entry;
    }

    pub fn unmap(&mut self) {
        self.entry = 0;
    }

    pub fn present(&self) -> bool {
        self.entry & (PageFlags::Present as usize) != 0
    }

    pub fn four_meg(&self) -> bool {
        self.entry & (PageFlags::FourMeg as usize) != 0
    }

    pub fn addr(&self) -> usize {
        self.entry & !(PageFlags::All as usize)
    }
}

type PT = [PageEntry<*mut u8>; 512];
type PD = [PageEntry<PT>; 512];
type PDPT = [PageEntry<PD>; 512];
type PML4 = [PageEntry<PDPT>; 512];

///
/// Memory map:
///
/// +-- kernel
/// |
/// V  0xFFFFF800_00000000 (8,192GB) blank
///    0xFFFFFF00_00000000   (512GB) recursive page tables
///    0xFFFFFF80_00000000   (512GB) init_pdpt
///    0xFFFFFFFF_80000000     (2GB)   -> identity mapped

unsafe fn usize2ref<T>(ptr: usize) -> &'static mut T {
    &mut *(ptr as *mut T)
}

const MMU_RECURSIVE_SLOT: usize = 510;

const KADDR_MMU_PT   : usize = 0xFFFF000000000000 + (MMU_RECURSIVE_SLOT<<39);
const KADDR_MMU_PD   : usize = KADDR_MMU_PT       + (MMU_RECURSIVE_SLOT<<30);
const KADDR_MMU_PDPT : usize = KADDR_MMU_PD       + (MMU_RECURSIVE_SLOT<<21);
const KADDR_MMU_PML4 : usize = KADDR_MMU_PDPT     + (MMU_RECURSIVE_SLOT<<12);

fn pml4()                 -> &'static mut PML4 { unsafe { usize2ref(KADDR_MMU_PML4) } }
fn pdpt<T>(ptr: *const T) -> &'static mut PDPT { unsafe { usize2ref(KADDR_MMU_PDPT + ((ptr as usize >> 27) & 0x00001FF000)) } }
fn pd<T>(ptr: *const T)   -> &'static mut PD   { unsafe { usize2ref(KADDR_MMU_PD   + ((ptr as usize >> 18) & 0x003FFFF000)) } }
fn pt<T>(ptr: *const T)   -> &'static mut PT   { unsafe { usize2ref(KADDR_MMU_PT   + ((ptr as usize >> 9)  & 0x7FFFFFF000)) } }

fn pml4_index<T>(ptr: *const T) -> usize { (ptr as usize >> 39) & 511 }
fn pdpt_index<T>(ptr: *const T) -> usize { (ptr as usize >> 30) & 511 }
fn pd_index<T>(ptr: *const T)   -> usize { (ptr as usize >> 21) & 511 }
fn pt_index<T>(ptr: *const T)   -> usize { (ptr as usize >> 12) & 511 }

fn pml4_entry<T>(ptr: *const T) -> &'static mut PageEntry<PDPT>    { &mut pml4()[pml4_index(ptr)] }
fn pdpt_entry<T>(ptr: *const T) -> &'static mut PageEntry<PD>      { &mut pdpt(ptr)[pdpt_index(ptr)] }
fn pd_entry<T>(ptr: *const T)   -> &'static mut PageEntry<PT>      { &mut pd(ptr)[pd_index(ptr)] }
fn pt_entry<T>(ptr: *const T)   -> &'static mut PageEntry<*mut u8> { &mut pt(ptr)[pt_index(ptr)] }

fn invlpg<T>(ptr: *const T) {
    unsafe {
        asm!("invlpg ($0)" :: "r"(ptr) : "memory" : "volatile")
    }
}

unsafe fn sysret<T>(ptr: *const T) -> ! {
    asm!("sysretq" :: "{rcx}" (ptr), "{r11}" (0) :: "volatile");
    unreachable!()
}

unsafe fn wrmsr(reg: u32, value: u64) {
    let value_hi = (value >> 32) as u32;
    let value_lo = value as u32;
    asm!("wrmsr" :: "{edx}" (value_hi), "{eax}" (value_lo), "{ecx}" (reg) :: "volatile");
}

fn map<T>(alloc: &Fn() -> usize, ptr: *const T, addr: usize, user: bool, writable: bool) {
    if pml4_entry(ptr).ensure_present(&alloc) {
        let pdpt = pdpt(ptr);
        for i in 0..512 {
            pdpt[i].unmap();
        }
    }

    if pdpt_entry(ptr).ensure_present(&alloc) {
        let pd = pd(ptr);
        for i in 0..512 {
            pd[i].unmap();
        }
    }

    let pd_entry = pd_entry(ptr);
    if pd_entry.four_meg() {
        panic!("didn't expect to map a page in the 4M region");
    }

    if pd_entry.ensure_present(&alloc) {
        let pt = pt(ptr);
        for i in 0..512 {
            pt[i].unmap();
        }
    }

    pt_entry(ptr).map(addr, user, writable);
    invlpg(ptr);
}

impl ArchProcess {
    pub fn new() -> ArchProcess {
        ArchProcess {
            dummy: 0
        }
    }

    pub fn kernel() -> ArchProcess {
        ArchProcess {
            dummy: 0
        }
    }
}

test! {
    fn check_pml4() {
        let entry = pml4_entry(0 as *mut u8);
        assert!(!entry.present());
        assert_eq!(0, entry.addr());
    }

    fn check_identity_mapping() {
        let two_meg = 2 * 1024 * 1024;
        let ptr: *const u8 = unsafe { phys_mem::phys2virt(two_meg) };
        assert!(pml4_entry(ptr).present());
        assert!(pdpt_entry(ptr).present());

        let pde = pd_entry(ptr);
        assert!(pde.present());
        assert!(pde.four_meg());
        assert_eq!(two_meg, pde.addr());
    }

    fn can_map() {
        let bitmap = PhysicalBitmap::parse_multiboot();
        let alloc = || bitmap.alloc_page().unwrap();
        let ptr1 = 0x1000 as *mut u16;
        let addr = bitmap.alloc_page().unwrap();
        map(&alloc, ptr1, addr, false, true);

        let ptr2 = unsafe { phys_mem::phys2virt(addr) };
        let sentinel = 0x55aa;
        unsafe {
            *ptr1 = sentinel;
            assert_eq!(sentinel, *ptr2);
        }
    }

    fn can_run_code_in_user_mode() {
        let bitmap = PhysicalBitmap::parse_multiboot();
        let alloc = || bitmap.alloc_page().unwrap();
        let code_ptr = 0x1000 as *mut u8;
        let code_addr = bitmap.alloc_page().unwrap();
        map(&alloc, code_ptr, code_addr, true, true);

        unsafe {
            const IA32_STAR: u32 = 0xC0000081;
            const IA32_LSTAR: u32 = 0xC0000082;
            wrmsr(IA32_STAR, 0x00100008_00000000);

            *code_ptr.offset(0) = 0x0f;
            *code_ptr.offset(1) = 0x05;

            match thread::setjmp() {
                Some(jmp_buf) => {
                    wrmsr(IA32_LSTAR, jmp_buf.rip as u64);
                    log!("sysret...");
                    sysret(code_ptr);
                }
                None => {
                    log!("...syscall");
                }
            }
        }
    }
}
