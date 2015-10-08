use alloc::arc::Arc;
use arch::cpu;
use arch::mmu;
use arch::pci;
use phys_mem::PhysicalBitmap;
use process::Process;
use virt_mem::VirtualTree;

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

pub unsafe fn vbe_set(xres: u16, yres: u16, bpp: u16) {
   vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_DISABLED);
   vbe_write(VBE_DISPI_INDEX_XRES, xres);
   vbe_write(VBE_DISPI_INDEX_YRES, yres);
   vbe_write(VBE_DISPI_INDEX_BPP, bpp);
   vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED);
}

test! {
    fn can_set_video_mode() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::for_kernel());
        let p = Process::new(phys, kernel_virt).unwrap();
        p.switch();

        unsafe {
            let bus_slot = pci::find(0x1234, 0x1111).unwrap();
            log!("Bochs VBE:");
            log!("  bus_slot = {:?}", bus_slot);

            let bar0 = pci::inl(bus_slot, 0, 0x10);
            let bar1 = pci::inl(bus_slot, 0, 0x14);
            let base_address = (bar1 as u64) << 32 | (bar0 as u64) & 0xffff_fff0;
            log!("  id is {:x}", vbe_read(VBE_DISPI_INDEX_ID));
            vbe_set(800, 600, 32);

            let lfb = p.map_phys::<u32>(base_address as usize, 800 * 600, false, true).unwrap();
            mmu::print_mapping(lfb.as_ptr());

            log!("  base address is {:x} phys, {:p} virt", base_address, lfb.as_ptr());
            log!("  video memory looks like {:x}", lfb[0]);
            for d in lfb {
                *d = 0xffffff;
            }
        }
    }
}
