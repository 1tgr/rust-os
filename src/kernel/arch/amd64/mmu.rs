use crate::arch::cpu;
use crate::phys_mem::{self, PhysicalBitmap};
use crate::ptr::Align;
use crate::spin::Mutex;
use alloc::sync::Arc;
use core::fmt::{Debug, Error, Formatter};
use core::marker::PhantomData;
use core::result;
use syscall::Result;

bitflags! {
    struct PageFlags: usize {
        const PAGE_PRESENT = 0x001;
        const PAGE_WRITABLE = 0x002;
        const PAGE_USER = 0x004;
        const PAGE_WRITETHROUGH = 0x008;
        const PAGE_NOCACHE = 0x010;
        const PAGE_ACCESSED = 0x020;
        const PAGE_DIRTY = 0x040; // PTE only
        const PAGE_BIG = 0x080; // PDE only
        const PAGE_GLOBAL = 0x100; // PTE only
    }
}

#[repr(C)]
struct PageEntry<T> {
    entry: usize,
    phantom: PhantomData<T>,
}

fn join(addr: usize, flags: PageFlags) -> usize {
    addr & !PageFlags::all().bits | flags.bits
}

impl<T> PageEntry<T> {
    pub fn entry(&self) -> (usize, PageFlags) {
        (
            self.entry & !PageFlags::all().bits,
            PageFlags::from_bits_truncate(self.entry),
        )
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

    pub fn ensure_present(&mut self, bitmap: &PhysicalBitmap) -> Result<()> {
        let mut flags = self.flags();
        if flags.contains(PageFlags::PAGE_PRESENT) {
            assert!(self.addr() != 0);
        } else {
            assert!(self.addr() == 0);
            flags.insert(PageFlags::PAGE_PRESENT | PageFlags::PAGE_WRITABLE | PageFlags::PAGE_USER);

            let addr = bitmap.alloc_zeroed_page()?;
            assert!(Align::is_aligned(addr, phys_mem::PAGE_SIZE));
            self.entry = join(addr, flags);
        }

        Ok(())
    }

    pub fn map(&mut self, addr: Option<usize>, user: bool, writable: bool) {
        let mut flags = PageFlags::empty();
        if addr.is_some() {
            flags.insert(PageFlags::PAGE_PRESENT);
        }

        if user {
            flags.insert(PageFlags::PAGE_USER);
        }

        if writable {
            flags.insert(PageFlags::PAGE_WRITABLE);
        }

        self.entry = if let Some(addr) = addr {
            assert!(Align::is_aligned(addr, phys_mem::PAGE_SIZE));
            join(addr, flags | PageFlags::PAGE_PRESENT)
        } else {
            join(0, flags)
        }
    }

    pub fn present(&self) -> bool {
        self.flags().contains(PageFlags::PAGE_PRESENT)
    }

    pub fn big(&self) -> bool {
        self.flags().contains(PageFlags::PAGE_BIG)
    }
}

impl<T> Debug for PageEntry<T> {
    fn fmt(&self, fmt: &mut Formatter) -> result::Result<(), Error> {
        let (addr, flags) = self.entry();
        write!(fmt, "{{ addr = {:-16x}, flags = ", addr)?;

        static ALL_FLAGS: &'static [(&'static str, PageFlags)] = &[
            ("G", PageFlags::PAGE_GLOBAL),
            ("B", PageFlags::PAGE_BIG),
            ("D", PageFlags::PAGE_DIRTY),
            ("A", PageFlags::PAGE_ACCESSED),
            ("C", PageFlags::PAGE_NOCACHE),
            ("T", PageFlags::PAGE_WRITETHROUGH),
            ("U", PageFlags::PAGE_USER),
            ("W", PageFlags::PAGE_WRITABLE),
            ("P", PageFlags::PAGE_PRESENT),
        ];

        for &(s, flag) in ALL_FLAGS.iter() {
            let s = if flags.contains(flag) { s } else { "." };
            fmt.write_str(s)?;
        }

        write!(fmt, " }} @ {:p}", &self.entry)
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

const KADDR_MMU_PT: usize = 0xFFFF000000000000 + (MMU_RECURSIVE_SLOT << 39);
const KADDR_MMU_PD: usize = KADDR_MMU_PT + (MMU_RECURSIVE_SLOT << 30);
const KADDR_MMU_PDPT: usize = KADDR_MMU_PD + (MMU_RECURSIVE_SLOT << 21);
const KADDR_MMU_PML4: usize = KADDR_MMU_PDPT + (MMU_RECURSIVE_SLOT << 12);

type PT = [PageEntry<*mut u8>; 512];
type PD = [PageEntry<PT>; 512];
type PDPT = [PageEntry<PD>; 512];
type PML4 = [PageEntry<PDPT>; 512];

fn pml4() -> &'static mut PML4 {
    unsafe { usize2ref(KADDR_MMU_PML4) }
}
fn pdpt<T>(ptr: *const T) -> &'static mut PDPT {
    unsafe { usize2ref(KADDR_MMU_PDPT + ((ptr as usize >> 27) & 0x00001FF000)) }
}
fn pd<T>(ptr: *const T) -> &'static mut PD {
    unsafe { usize2ref(KADDR_MMU_PD + ((ptr as usize >> 18) & 0x003FFFF000)) }
}
fn pt<T>(ptr: *const T) -> &'static mut PT {
    unsafe { usize2ref(KADDR_MMU_PT + ((ptr as usize >> 9) & 0x7FFFFFF000)) }
}

fn pml4_index<T>(ptr: *const T) -> usize {
    (ptr as usize >> 39) & 511
}
fn pdpt_index<T>(ptr: *const T) -> usize {
    (ptr as usize >> 30) & 511
}
fn pd_index<T>(ptr: *const T) -> usize {
    (ptr as usize >> 21) & 511
}
fn pt_index<T>(ptr: *const T) -> usize {
    (ptr as usize >> 12) & 511
}

fn pml4_entry<T>(ptr: *const T) -> &'static mut PageEntry<PDPT> {
    &mut pml4()[pml4_index(ptr)]
}
fn pdpt_entry<T>(ptr: *const T) -> &'static mut PageEntry<PD> {
    &mut pdpt(ptr)[pdpt_index(ptr)]
}
fn pd_entry<T>(ptr: *const T) -> &'static mut PageEntry<PT> {
    &mut pd(ptr)[pd_index(ptr)]
}
fn pt_entry<T>(ptr: *const T) -> &'static mut PageEntry<*mut u8> {
    &mut pt(ptr)[pt_index(ptr)]
}

pub fn print_mapping<T>(ptr: *const T) {
    let pml4_entry = pml4_entry(ptr);
    log!("[{:p}] PML4 = {:?}", ptr, pml4_entry);
    if pml4_entry.present() {
        let pdpt_entry = pdpt_entry(ptr);
        log!("[{:p}] PDPT = {:?}", ptr, pdpt_entry);
        if pdpt_entry.present() {
            let pd_entry = pd_entry(ptr);
            log!("[{:p}]   PD = {:?}", ptr, pd_entry);
            if pd_entry.present() && !pd_entry.big() {
                let pt_entry = pt_entry(ptr);
                log!("[{:p}]   PT = {:?}", ptr, pt_entry);
            }
        }
    }
}

extern "C" {
    static KERNEL_BASE: u8;
    static init_pml4: u8;
}

pub struct AddressSpace {
    mutex: Mutex<()>,
    bitmap: Arc<PhysicalBitmap>,
    cr3: usize,
}

impl AddressSpace {
    pub fn new(bitmap: Arc<PhysicalBitmap>) -> Result<AddressSpace> {
        let pml4_addr = bitmap.alloc_zeroed_page()?;
        let two_meg = 2 * 1024 * 1024;

        unsafe {
            let pml4: &mut PML4 = &mut phys_mem::phys2virt(pml4_addr);
            pml4[MMU_RECURSIVE_SLOT].map(Some(pml4_addr), false, true);

            let kernel_base_ptr = Align::down(&KERNEL_BASE as *const u8, phys_mem::PAGE_SIZE);
            for addr in (0..bitmap.total_bytes()).step_by(two_meg) {
                let ptr = kernel_base_ptr.offset(addr as isize);
                let pml4_index = pml4_index(ptr);
                let pdpt_index = pdpt_index(ptr);
                let pd_index = pd_index(ptr);

                let pml4_entry = &mut pml4[pml4_index];
                pml4_entry.ensure_present(&bitmap)?;

                let pdpt_entry = &mut pml4_entry.as_mut_ref()[pdpt_index];
                pdpt_entry.ensure_present(&bitmap)?;

                let pd_entry = &mut pdpt_entry.as_mut_ref()[pd_index];
                pd_entry.entry = join(
                    addr,
                    PageFlags::PAGE_PRESENT | PageFlags::PAGE_WRITABLE | PageFlags::PAGE_BIG,
                );
            }
        }

        Ok(Self {
            mutex: Mutex::new(()),
            bitmap,
            cr3: pml4_addr,
        })
    }

    pub unsafe fn switch(&self) {
        if cpu::read_cr3() != self.cr3 {
            let _x = lock!(self.mutex);
            cpu::write_cr3(self.cr3);
        }
    }

    pub unsafe fn map<T>(&self, ptr: *const T, addr: Option<usize>, user: bool, writable: bool) -> Result<()> {
        let _x = lock!(self.mutex);
        let pml4_entry = pml4_entry(ptr);
        let pdpt_entry = pdpt_entry(ptr);
        let pd_entry = pd_entry(ptr);
        let pt_entry = pt_entry(ptr);
        pml4_entry.ensure_present(&self.bitmap)?;
        pdpt_entry.ensure_present(&self.bitmap)?;
        assert!(!pd_entry.big());
        pd_entry.ensure_present(&self.bitmap)?;
        pt_entry.map(addr, user, writable);
        cpu::invlpg(ptr);
        Ok(())
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        if cpu::read_cr3() == self.cr3 {
            let cr3 = phys_mem::virt2phys(unsafe { &init_pml4 });
            unsafe { cpu::write_cr3(cr3) };
        }

        // TODO free memory
    }
}

#[cfg(feature = "test")]
pub mod test {
    use super::*;
    use crate::phys_mem::{self, PhysicalBitmap};
    use crate::ptr::Align;
    use alloc::sync::Arc;
    use core::intrinsics;
    use core::mem;

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
            assert!(pde.big());
            assert_eq!(two_meg, pde.addr());
        }

        fn can_switch() {
            let bitmap = Arc::new(PhysicalBitmap::machine());
            let address_space = AddressSpace::new(bitmap).unwrap();
            unsafe { address_space.switch() };
        }

        fn can_map_user() {
            let bitmap = Arc::new(PhysicalBitmap::machine());
            let ptr1 = 0x1000 as *mut u16;
            let addr = bitmap.alloc_page().unwrap();
            let address_space = AddressSpace::new(bitmap).unwrap();
            unsafe {
                address_space.switch();
                address_space.map(ptr1, Some(addr), false, true).unwrap();

                let ptr2 = phys_mem::phys2virt(addr);
                let sentinel = 0x55aa;
                intrinsics::volatile_store(ptr1, sentinel);
                assert_eq!(sentinel, intrinsics::volatile_load(ptr2));
            }
        }

        fn can_map_kernel() {
            let bitmap = Arc::new(PhysicalBitmap::machine());
            let two_meg = 2 * 1024 * 1024;

            let ptr1: *mut u16 =
                unsafe {
                    let kernel_base_ptr = &KERNEL_BASE as *const u8;
                    let ptr1 = Align::up(kernel_base_ptr.offset(bitmap.total_bytes() as isize), two_meg);
                    mem::transmute(ptr1)
                };

            let addr = bitmap.alloc_page().unwrap();
            let address_space = AddressSpace::new(bitmap).unwrap();
            unsafe {
                address_space.switch();
                address_space.map(ptr1, Some(addr), false, true).unwrap();

                let ptr2 = phys_mem::phys2virt(addr);
                let sentinel = 0x55aa;
                intrinsics::volatile_store(ptr1, sentinel);
                assert_eq!(sentinel, intrinsics::volatile_load(ptr2));
            }
        }
    }
}
