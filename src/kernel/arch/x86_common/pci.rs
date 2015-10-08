use arch::cpu;

fn address((bus, slot): (u8, u8), func: u8, offset: u8) -> u32 {
    0x80000000
    | (bus as u32) << 16
    | (slot as u32) << 11
    | (func as u32) << 8
    | (offset as u32) & 0xfc
}

pub unsafe fn inw(bus_slot: (u8, u8), func: u8, offset: u8) -> u16 {
    let address = address(bus_slot, func, offset);
    cpu::outl(0xcf8, address);
    (cpu::inl(0xcfc) >> ((offset & 2) * 8)) as u16
}

pub unsafe fn inl(bus_slot: (u8, u8), func: u8, offset: u8) -> u32 {
    let address = address(bus_slot, func, offset);
    cpu::outl(0xcf8, address);
    cpu::inl(0xcfc)
}

pub unsafe fn find(vendor: u16, device: u16) -> Option<(u8, u8)> {
    for bus in 0..256 {
        for slot in 0..32 {
            let bus_slot = (bus as u8, slot);
            let found_vendor = inw(bus_slot, 0, 0);
            if found_vendor == 0xffff {
                break;
            }

            if found_vendor == vendor && inw(bus_slot, 0, 2) == device {
                return Some(bus_slot);
            }
        }
    }

    None
}
