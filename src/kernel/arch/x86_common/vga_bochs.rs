use arch::cpu;
use arch::mmu;
use arch::pci;
use core::mem;
use process::Process;
use syscall::{ErrNum,Result};

const VBE_DISPI_IOPORT_INDEX: u16 = 0x1ce;
const VBE_DISPI_IOPORT_DATA: u16 = 0x1cf;
const VBE_DISPI_INDEX_ID: u16 = 0;
const VBE_DISPI_INDEX_XRES: u16 = 1;
const VBE_DISPI_INDEX_YRES: u16 = 2;
const VBE_DISPI_INDEX_BPP: u16 = 3;
const VBE_DISPI_INDEX_ENABLE: u16 = 4;
const VBE_DISPI_DISABLED: u16 = 0;
const VBE_DISPI_ENABLED: u16 = 1;
const VBE_DISPI_LFB_ENABLED: u16 = 64;

unsafe fn vbe_read(index: u16) -> u16 {
   cpu::outw(VBE_DISPI_IOPORT_INDEX, index);
   cpu::inw(VBE_DISPI_IOPORT_DATA)
}

unsafe fn vbe_write(index: u16, value: u16) {
   cpu::outw(VBE_DISPI_IOPORT_INDEX, index);
   cpu::outw(VBE_DISPI_IOPORT_DATA, value);
}

pub unsafe fn init<T>(p: &Process, xres: u16, yres: u16, bpp: u16) -> Result<&mut [T]> {
    let bus_slot =
        try!(pci::find(0x1234, 0x1111)              // QEmu
            .or_else(|| pci::find(0x80EE, 0xBEEF))  // VirtualBox
            .ok_or(ErrNum::NotSupported));

    log!("Bochs VBE:");
    log!("  bus_slot = {:?}", bus_slot);

    let bar0 = pci::inl(bus_slot, 0, 0x10);
    let bar1 = pci::inl(bus_slot, 0, 0x14);
    let base_address = (bar1 as u64) << 32 | (bar0 as u64) & 0xffff_fff0;
    log!("  id is {:x}", vbe_read(VBE_DISPI_INDEX_ID));
    vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_DISABLED);
    vbe_write(VBE_DISPI_INDEX_XRES, xres);
    vbe_write(VBE_DISPI_INDEX_YRES, yres);
    vbe_write(VBE_DISPI_INDEX_BPP, bpp);
    vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED);

    let len = (xres as usize * yres as usize * bpp as usize) / (8 * mem::size_of::<T>());
    let lfb = p.map_phys::<T>(base_address as usize, len, false, true).unwrap();
    mmu::print_mapping(lfb.as_ptr());
    log!("  base address is {:x} phys, {:p} virt", base_address, lfb.as_ptr());
    Ok(lfb)
}

#[cfg(feature = "test")]
pub mod test {
    use alloc::arc::Arc;
    use phys_mem::PhysicalBitmap;
    use process::Process;
    use super::*;
    use virt_mem::VirtualTree;

    test! {
        fn can_clear_screen() {
            let phys = Arc::new(PhysicalBitmap::parse_multiboot());
            let kernel_virt = Arc::new(VirtualTree::for_kernel());
            let p = Process::new(phys, kernel_virt).unwrap();
            p.switch();

            let lfb = unsafe { init(&p, 800, 600, 32).unwrap() };
            for d in lfb {
                *d = 0xffffff;
            }
        }
    }
}
