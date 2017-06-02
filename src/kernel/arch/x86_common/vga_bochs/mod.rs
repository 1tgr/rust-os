use arch::cpu;
use arch::pci;
use process;
use spin::{StaticMutex,STATIC_MUTEX_INIT};
use syscall::{ErrNum,Result};

static MUTEX: StaticMutex = STATIC_MUTEX_INIT;

const VBE_DISPI_IOPORT_INDEX: u16 = 0x1ce;
const VBE_DISPI_IOPORT_DATA: u16 = 0x1cf;
const VBE_DISPI_INDEX_XRES: u16 = 1;
const VBE_DISPI_INDEX_YRES: u16 = 2;
const VBE_DISPI_INDEX_BPP: u16 = 3;
const VBE_DISPI_INDEX_ENABLE: u16 = 4;
const VBE_DISPI_DISABLED: u16 = 0;
const VBE_DISPI_ENABLED: u16 = 1;
const VBE_DISPI_LFB_ENABLED: u16 = 64;

unsafe fn vbe_write(index: u16, value: u16) {
   cpu::outw(VBE_DISPI_IOPORT_INDEX, index);
   cpu::outw(VBE_DISPI_IOPORT_DATA, value);
}

pub fn init(xres: u16, yres: u16, bpp: u8) -> Result<&'static mut [u8]> {
    let _d = lock!(MUTEX);
    unsafe {
        let bus_slot =
            pci::find(0x1234, 0x1111)                   // QEmu
                .or_else(|| pci::find(0x80EE, 0xBEEF))  // VirtualBox
                .ok_or(ErrNum::NotSupported)?;

        let bar0 = pci::inl(bus_slot, 0, 0x10);
        let bar1 = pci::inl(bus_slot, 0, 0x14);
        let base_address = (bar1 as u64) << 32 | (bar0 as u64) & 0xffff_fff0;
        vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_DISABLED);
        vbe_write(VBE_DISPI_INDEX_XRES, xres);
        vbe_write(VBE_DISPI_INDEX_YRES, yres);
        vbe_write(VBE_DISPI_INDEX_BPP, bpp as u16);
        vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED);

        let len = (xres as usize * yres as usize * bpp as usize) / 8;
        process::map_phys(base_address as usize, len, true, true)
    }
}
