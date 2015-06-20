pub const IA32_STAR: u32 = 0xC0000081;
pub const IA32_LSTAR: u32 = 0xC0000082;

pub fn invlpg<T>(ptr: *const T) {
    unsafe { asm!("invlpg ($0)" :: "r"(ptr) : "memory" : "volatile") }
}

pub unsafe fn sysret<T, U>(rip: *const T, rsp: *const U) -> ! {
    asm!("cli ; mov $0, %rsp ; sysretq" :: "r"(rsp), "{rcx}" (rip), "{r11}" (0) :: "volatile");
    unreachable!()
}

pub unsafe fn wrmsr(reg: u32, value: u64) {
    let value_hi = (value >> 32) as u32;
    let value_lo = value as u32;
    asm!("wrmsr" :: "{edx}" (value_hi), "{eax}" (value_lo), "{ecx}" (reg) :: "volatile");
}

pub fn read_cr2<T>() -> *mut T {
    let cr2;
    unsafe { asm!("mov %cr2, $0" : "=r"(cr2)) };
    cr2
}

pub fn read_cr3() -> usize {
    let cr3;
    unsafe { asm!("mov %cr3, $0" : "=r"(cr3)) };
    cr3
}

pub unsafe fn write_cr3(addr: usize) {
    asm!("mov $0, %cr3" :: "r"(addr) : "memory" : "volatile");
}

pub unsafe fn int(num: u8) {
    asm!("int $0" :: "N"(num) : "memory");
}
