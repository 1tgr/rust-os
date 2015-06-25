use ::arch::cpu;
use ::phys_mem::{self,PhysicalBitmap};
use ::ptr::Align;
use ::thread;
use spin::Mutex;
use std::fmt::{Debug,Error,Formatter};
use std::intrinsics;
use std::marker::PhantomData;
use std::sync::Arc;

bitflags! {
    flags PageFlags: usize {
        const PAGE_PRESENT = 0x001,
        const PAGE_WRITABLE = 0x002,
        const PAGE_USER = 0x004,
        const PAGE_WRITETHROUGH = 0x008,
        const PAGE_NOCACHE = 0x010,
        const PAGE_ACCESSED = 0x020,
        const PAGE_DIRTY = 0x040, // PTE only
        const PAGE_FOURMEG = 0x080, // PDE only
        const PAGE_GLOBAL = 0x100 // PTE only
    }
}

#[repr(C)]
pub struct PageEntry<T> {
    entry: usize,
    phantom: PhantomData<T>
}

fn join(addr: usize, flags: PageFlags) -> usize {
    addr & !PageFlags::all().bits | flags.bits
}

impl<T> PageEntry<T> {
    pub fn entry(&self) -> (usize, PageFlags) {
        (self.entry & !PageFlags::all().bits, PageFlags::from_bits_truncate(self.entry))
    }

    pub fn addr(&self) -> usize {
        let (addr, _) = self.entry();
        addr
    }

    pub fn flags(&self) -> PageFlags {
        let (_, flags) = self.entry();
        flags
    }

    pub unsafe fn as_mut_ref(&self) -> &'static mut T {
        phys_mem::phys2virt(self.addr())
    }

    pub unsafe fn as_ref(&self) -> &'static T {
        self.as_mut_ref()
    }

    pub fn ensure_present(&mut self, bitmap: &PhysicalBitmap) -> Result<(), &'static str> {
        let mut flags = self.flags();
        if !flags.contains(PAGE_PRESENT) {
            flags.insert(PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER);

            let addr = try!(bitmap.alloc_page());
            assert!(Align::is_aligned(addr, phys_mem::PAGE_SIZE));
            self.entry = join(addr, flags);
        }

        Ok(())
    }

    pub fn map(&mut self, addr: usize, user: bool, writable: bool) {
        let mut flags = PAGE_PRESENT;
        if user {
            flags.insert(PAGE_USER);
        }

        if writable {
            flags.insert(PAGE_WRITABLE);
        }

        assert!(Align::is_aligned(addr, phys_mem::PAGE_SIZE));
        self.entry = join(addr, flags);
    }

    pub fn unmap(&mut self) {
        self.entry = 0;
    }

    pub fn present(&self) -> bool {
        self.flags().contains(PAGE_PRESENT)
    }

    pub fn four_meg(&self) -> bool {
        self.flags().contains(PAGE_FOURMEG)
    }
}

impl<T> Debug for PageEntry<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let (addr, flags) = self.entry();
        try!(write!(fmt, "{{ addr = {:-16x}, flags = ", addr));

        static ALL_FLAGS: &'static [(&'static str, PageFlags)] = &[
            ("G", PAGE_GLOBAL),
            ("4", PAGE_FOURMEG),
            ("D", PAGE_DIRTY),
            ("A", PAGE_ACCESSED),
            ("C", PAGE_NOCACHE),
            ("T", PAGE_WRITETHROUGH),
            ("U", PAGE_USER),
            ("W", PAGE_WRITABLE),
            ("P", PAGE_PRESENT)
        ];

        for &(s, flag) in ALL_FLAGS.iter() {
            let s = if flags.contains(flag) { s } else { "." };
            try!(fmt.write_str(s));
        }

        fmt.write_str(" }")
    }
}

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

pub type PT = [PageEntry<*mut u8>; 512];
pub type PD = [PageEntry<PT>; 512];
pub type PDPT = [PageEntry<PD>; 512];
pub type PML4 = [PageEntry<PDPT>; 512];

fn pml4()                 -> &'static mut PML4 { unsafe { usize2ref(KADDR_MMU_PML4) } }
fn pdpt<T>(ptr: *const T) -> &'static mut PDPT { unsafe { usize2ref(KADDR_MMU_PDPT + ((ptr as usize >> 27) & 0x00001FF000)) } }
fn pd<T>(ptr: *const T)   -> &'static mut PD   { unsafe { usize2ref(KADDR_MMU_PD   + ((ptr as usize >> 18) & 0x003FFFF000)) } }
fn pt<T>(ptr: *const T)   -> &'static mut PT   { unsafe { usize2ref(KADDR_MMU_PT   + ((ptr as usize >> 9)  & 0x7FFFFFF000)) } }

fn pml4_index<T>(ptr: *const T) -> usize { (ptr as usize >> 39) & 511 }
fn pdpt_index<T>(ptr: *const T) -> usize { (ptr as usize >> 30) & 511 }
fn pd_index<T>(ptr: *const T)   -> usize { (ptr as usize >> 21) & 511 }
fn pt_index<T>(ptr: *const T)   -> usize { (ptr as usize >> 12) & 511 }

pub fn pml4_entry<T>(ptr: *const T) -> &'static mut PageEntry<PDPT>    { &mut pml4()[pml4_index(ptr)] }
pub fn pdpt_entry<T>(ptr: *const T) -> &'static mut PageEntry<PD>      { &mut pdpt(ptr)[pdpt_index(ptr)] }
pub fn pd_entry<T>(ptr: *const T)   -> &'static mut PageEntry<PT>      { &mut pd(ptr)[pd_index(ptr)] }
pub fn pt_entry<T>(ptr: *const T)   -> &'static mut PageEntry<*mut u8> { &mut pt(ptr)[pt_index(ptr)] }

fn recursive_pml4_addr() -> usize {
    pml4()[MMU_RECURSIVE_SLOT].addr()
}

extern {
    static KERNEL_BASE: u8;
    static init_pml4: u8;
}

pub struct AddressSpace {
    mutex: Mutex<()>,
    bitmap: Arc<PhysicalBitmap>,
    cr3: usize
}

impl AddressSpace {
    pub fn new(bitmap: Arc<PhysicalBitmap>) -> Result<AddressSpace, &'static str> {
        let pml4_addr = try!(bitmap.alloc_page());
        let kernel_base_ptr = &KERNEL_BASE as *const u8;
        let four_meg = 4 * 1024 * 1024;

        unsafe {
            let pml4: &mut PML4 = &mut phys_mem::phys2virt(pml4_addr);
            pml4[MMU_RECURSIVE_SLOT].map(pml4_addr, false, true);

            let pdpt_identity_entry: &mut PageEntry<PDPT> = &mut pml4[pml4_index(kernel_base_ptr)];
            try!(pdpt_identity_entry.ensure_present(&bitmap));

            let pdpt_identity: &mut PDPT = pdpt_identity_entry.as_mut_ref();
            for i in 0..2 {
                pdpt_identity[pdpt_index(kernel_base_ptr) + i].entry = join(i * four_meg, PAGE_PRESENT | PAGE_WRITABLE | PAGE_FOURMEG);
            }
        }

        Ok(AddressSpace {
            mutex: Mutex::new(()),
            bitmap: bitmap,
            cr3: pml4_addr
        })
    }

    pub fn switch(&self) {
        let _ = self.mutex.lock();
        log!("switch: {:x} -> {:x}", recursive_pml4_addr(), self.cr3);
        unsafe { cpu::write_cr3(self.cr3) };
    }

    pub fn map<T>(&self, ptr: *const T, addr: usize, user: bool, writable: bool) -> Result<(), &'static str> {
        let _ = self.mutex.lock();
        assert_eq!(self.cr3, recursive_pml4_addr());

        try!(pml4_entry(ptr).ensure_present(&self.bitmap));
        try!(pdpt_entry(ptr).ensure_present(&self.bitmap));

        let pd_entry = pd_entry(ptr);
        if pd_entry.four_meg() {
            panic!("didn't expect to map a page in the 4M region");
        }

        try!(pd_entry.ensure_present(&self.bitmap));
        pt_entry(ptr).map(addr, user, writable);
        cpu::invlpg(ptr);
        Ok(())
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        if cpu::read_cr3() == self.cr3 {
            let cr3 = phys_mem::virt2phys(&init_pml4);
            log!("drop: {:x} -> {:x}", self.cr3, cr3);
            unsafe { cpu::write_cr3(cr3) };
        }

        // TODO free memory
    }
}

pub unsafe fn call_user_mode<T, U>(rip: *const T, rsp: *const U) {
    static mut KERNEL_RSP: i64 = 0;
    cpu::wrmsr(cpu::IA32_STAR, 0x00100008_00000000);

    match thread::setjmp() {
        Some(jmp_buf) => {
            cpu::wrmsr(cpu::IA32_LSTAR, jmp_buf.rip as u64);
            KERNEL_RSP = jmp_buf.rsp;
            log!("sysret...");
            cpu::sysret(rip, rsp);
        }
        None => {
            asm!("mov $0, %rsp" :: "r"(KERNEL_RSP) : "memory" : "volatile");
            log!("...syscall");
        }
    }
}

test! {
    fn check_pml4() {
        let entry = pml4_entry(0 as *const u8);
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

    fn can_switch() {
        let bitmap = Arc::new(PhysicalBitmap::parse_multiboot());
        let address_space = AddressSpace::new(bitmap).unwrap();
        address_space.switch();
    }

    fn can_map() {
        let bitmap = Arc::new(PhysicalBitmap::parse_multiboot());
        let ptr1 = 0x1000 as *mut u16;
        let addr = bitmap.alloc_page().unwrap();
        let address_space = AddressSpace::new(bitmap).unwrap();
        address_space.switch();
        address_space.map(ptr1, addr, false, true).unwrap();

        let ptr2 = unsafe { phys_mem::phys2virt(addr) };
        let sentinel = 0x55aa;
        unsafe {
            intrinsics::volatile_store(ptr1, sentinel);
            assert_eq!(sentinel, intrinsics::volatile_load(ptr2));
        }
    }

    fn can_run_code_in_user_mode() {
        let bitmap = Arc::new(PhysicalBitmap::parse_multiboot());
        assert_eq!(4096, phys_mem::PAGE_SIZE);

        let code_ptr = 0x1000 as *mut u8;
        let code_addr = bitmap.alloc_page().unwrap();
        let stack_ptr = 0x2000 as *mut u8;
        let stack_addr = bitmap.alloc_page().unwrap();
        let address_space = AddressSpace::new(bitmap).unwrap();
        address_space.switch();
        address_space.map(code_ptr, code_addr, true, true).unwrap();
        address_space.map(stack_ptr, stack_addr, true, true).unwrap();
        unsafe {
            *code_ptr.offset(0) = 0x0f;
            *code_ptr.offset(1) = 0x05;
            call_user_mode(code_ptr, stack_ptr.offset(4096))
        }
    }
}

