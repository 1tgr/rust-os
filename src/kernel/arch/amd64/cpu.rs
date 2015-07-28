#[repr(C)]
#[derive(Debug)]
pub struct Regs {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub error: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64
}

#[repr(C, packed)]
#[derive(Default)]
pub struct Tss {
    pub reserved1: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    pub reserved2: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    pub reserved3: u64,
    pub iopm_len: u16,
    pub reserved4: u16
}

#[repr(C, packed)]
#[derive(Default)]
pub struct DescriptorExtra {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_mid: u8,
    pub access: u8,
    pub limit_high_and_flags: u8,
    pub base_high: u8,
    pub base_extra: u32,
    pub reserved: u32
}

#[repr(C, packed)]
#[derive(Default)]
pub struct InterruptDescriptor {
    pub offset_low: u16,
    pub selector: u16,
    pub reserved1: u8,
    pub type_attr: u8,
    pub offset_high: u16,
    pub offset_extra: u32,
    pub reserved2: u32
}

#[repr(C, packed)]
pub struct Dtr {
    pub limit: u16,
    pub base: u64
}

pub const IA32_STAR: u32 = 0xC0000081;
pub const IA32_LSTAR: u32 = 0xC0000082;
pub const IA32_SFMASK: u32 = 0xC0000084;

extern {
    pub fn lidt(ptr: &Dtr); // I don't know why I can't get lidt to work via inline asm
}

pub fn invlpg<T>(ptr: *const T) {
    unsafe { asm!("invlpg ($0)" :: "r"(ptr) : "memory" : "volatile") }
}

pub unsafe fn sysret<T, U>(rip: *const T, rsp: *const U, rflags: u64) -> ! {
    asm!("cli ; mov $0, %rsp ; sysretq" :: "r"(rsp), "{rcx}" (rip), "{r11}" (rflags) :: "volatile");
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

pub unsafe fn ltr(selector: u16) {
    asm!("ltr $0" :: "r"(selector) :: "volatile");
}

pub unsafe fn sti() {
    asm!("sti" :::: "volatile");
}

pub fn interrupts_enabled() -> bool {
    let rflags : usize;
    unsafe { asm!("pushfq ; pop $0" : "=r"(rflags)) };
    (rflags & (1 << 9)) != 0
}

pub fn current_frame() -> *const usize {
    let rbp;
    unsafe { asm!("mov %rbp, $0" : "=r"(rbp)) };
    rbp
}

pub unsafe fn outb(port: u16, val: u8) {
    asm!("outb %al, %dx" : : "{dx}" (port), "{al}" (val) : : "volatile");
}

pub unsafe fn inb(port: u16) -> u8 {
	let ret : u8;
	asm!("inb %dx, %al" : "={al}" (ret) : "{dx}" (port) : : "volatile");
	return ret;
}

