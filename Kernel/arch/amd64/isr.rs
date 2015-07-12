use ::arch::cpu::{self,DescriptorExtra,Dtr,InterruptDescriptor,Regs,Tss};
use ::arch::mmu;
use ::arch::x86_common::io;
use ::ptr;
use std::default::Default;
use std::mem;
use lazy_static::once::{self,Once};

extern {
    static kernel_start: u8;
    static kernel_end: u8;
    static GDT: u8;
    static mut GDT_TSS: DescriptorExtra;
    static mut TSS: Tss;
    static TSSEnd: u8;
    static mut IDT: [InterruptDescriptor; 49];
    static IDTEnd: u8;
    static interrupt_handlers: [u64; 49];
    static interrupt_handlers_end: u8;
}

static mut syscall_stack: [u8; 4096] = [0; 4096];

const PIC1: u16 = 0x20; // IO base address for master PIC
const PIC2: u16 = 0xA0; // IO base address for slave PIC
const PIC1_COMMAND: u16 = PIC1;
const PIC1_DATA: u16 = PIC1 + 1;
const PIC2_COMMAND: u16 = PIC2;
const PIC2_DATA: u16 = PIC2 + 1;

macro_rules! assert_size {
	($value:expr, $expected_end:expr) => ({
        let value = $value;
        let expected_end = $expected_end;
        let expected_len = ptr::bytes_between(value as *const _ as *const u8, expected_end as *const _ as *const u8);
        assert_eq!(mem::size_of_val(value), expected_len);
    })
}

macro_rules! assert_len {
	($slice:expr, $expected_end:expr) => ({
        let slice = $slice;
        let expected_end = $expected_end;
        let expected_len = ptr::bytes_between(slice.as_ptr() as *const u8, expected_end as *const u8) / mem::size_of_val(&*slice.as_ptr());
        assert_eq!(expected_len, slice.len());
    })
}

pub fn init_once() {
    static ONCE: Once = once::ONCE_INIT;
    ONCE.call_once(|| unsafe {
        assert_size!(&TSS, &TSSEnd);
        assert_len!(&interrupt_handlers, &interrupt_handlers_end);
        assert_len!(&IDT, &IDTEnd);
        assert_eq!(104, mem::size_of::<Tss>());
        assert_eq!(10, mem::size_of::<Dtr>());

        let tss_selector = ptr::bytes_between(&GDT, &GDT_TSS as *const _ as *const u8) as u16;
        assert_eq!(0x38, tss_selector);

        let tss_ptr = &TSS as *const _ as usize;
        GDT_TSS = Default::default();
        GDT_TSS.limit_low = mem::size_of::<Tss>() as u16;
        GDT_TSS.base_low = tss_ptr as u16;
        GDT_TSS.base_mid = (tss_ptr >> 16) as u8;
        GDT_TSS.base_high = (tss_ptr >> 24) as u8;
        GDT_TSS.base_extra = (tss_ptr >> 32) as u32;
        GDT_TSS.access = 0x89;
        GDT_TSS.limit_high_and_flags = 0x10;

        TSS = Default::default();
        TSS.rsp0 = (&syscall_stack as *const _ as usize + mem::size_of_val(&syscall_stack)) as u64;
        TSS.iopm_len = mem::size_of::<Tss>() as u16;

        cpu::ltr(tss_selector);

        for (handler_ptr, desc) in interrupt_handlers.iter().zip(IDT.iter_mut()) {
            let handler_ptr: u64 = *handler_ptr;
            *desc = Default::default();
            desc.offset_low = handler_ptr as u16;
            desc.offset_high = (handler_ptr >> 16) as u16;
            desc.offset_extra = (handler_ptr >> 32) as u32;
            desc.type_attr = 0x8e; // 32-bit interrupt gate: 0x8E ( P=1, DPL=00b, S=0, type=1110b => type_attr=1000_1110b=0x8E)
            desc.selector = 0x08;
        }

        let idtr = Dtr {
            limit: mem::size_of_val(&IDT) as u16,
            base: IDT.as_ptr() as u64
        };

        cpu::lidt(&idtr); 

        const ICW1_ICW4: u8 = 0x01;         /* ICW4 (not) needed */
        const ICW1_INIT: u8 = 0x10;         /* Initialization - required! */
        const ICW4_8086: u8 = 0x01;         /* 8086/88 (MCS-80/85) mode */

        const OFFSET1: u8 = 32;
        const OFFSET2: u8 = 40;

        let a1 = io::inb(PIC1_DATA);                        // save masks
        let a2 = io::inb(PIC2_DATA);

        io::outb(PIC1_COMMAND, ICW1_INIT+ICW1_ICW4);  // starts the initialization sequence (in cascade mode)
        io::outb(PIC2_COMMAND, ICW1_INIT+ICW1_ICW4);
        io::outb(PIC1_DATA, OFFSET1);                 // ICW2: Master PIC vector offset
        io::outb(PIC2_DATA, OFFSET2);                 // ICW2: Slave PIC vector offset
        io::outb(PIC1_DATA, 4);                       // ICW3: tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
        io::outb(PIC2_DATA, 2);                       // ICW3: tell Slave PIC its cascade identity (0000 0010)

        io::outb(PIC1_DATA, ICW4_8086);
        io::outb(PIC2_DATA, ICW4_8086);

        io::outb(PIC1_DATA, a1);   // restore saved masks.
        io::outb(PIC2_DATA, a2);

        cpu::sti();
    });
}

#[no_mangle]
pub extern fn interrupt(num: u8, _: &Regs) {
    log!("interrupt {}", num);
}

#[no_mangle]
pub extern fn irq(num: u8, _: &Regs) {
    unsafe {
        const PIC_EOI: u8 = 0x20; // End-of-interrupt command code

        if num != 0 {
            log!("irq {}", num);
        }

        if num >= 8 {
            io::outb(PIC2_COMMAND, PIC_EOI);
        }

        io::outb(PIC1_COMMAND, PIC_EOI);
    }
}

#[no_mangle]
pub extern fn exception(num: u8, regs: &Regs) {
    let cr2: *const u8 = cpu::read_cr2();
    log!("exception {}: error=0x{:x}  cr2={:p}", num, regs.error, cr2);
    log!("ss:rsp={:x}:{:-16x}  cs:rip={:x}:{:-16x} rflags={:x}", regs.ss, regs.rsp, regs.cs, regs.rip, regs.rflags);
    log!("rax={:-16x} rbx={:-16x} rcx={:-16x} rdx={:-16x}", regs.rax, regs.rbx, regs.rcx, regs.rdx);
    log!("rbp={:-16x} rdi={:-16x} rsi={:-16x}", regs.rbp, regs.rdi, regs.rsi);
    log!(" r8={:-16x}  r9={:-16x} r10={:-16x} r11={:-16x}", regs.r8, regs.r9, regs.r10, regs.r11);
    log!("r12={:-16x} r12={:-16x} r14={:-16x} r15={:-16x}", regs.r12, regs.r13, regs.r14, regs.r15);
    log!("");

    if num == 14 {
        log!("page fault: {} {} in {} mode",
             if (regs.error & 1) != 0 { "protection violation" } else { "page not present" },
             if (regs.error & 2) != 0 { "writing" } else { "reading" },
             if (regs.error & 4) != 0 { "user" } else { "kernel" });

        log!("cr3 = {:x}", cpu::read_cr3());

        let pml4_entry = mmu::pml4_entry(cr2);
        log!("PML4 = {:?}", pml4_entry);
        if pml4_entry.present() {
            let pdpt_entry = mmu::pdpt_entry(cr2);
            log!("PDPT = {:?}", pdpt_entry);
            if pdpt_entry.present() {
                let pd_entry = mmu::pd_entry(cr2);
                log!("  PD = {:?}", pd_entry);
                if pd_entry.present() {
                    let pt_entry = mmu::pt_entry(cr2);
                    log!("  PT = {:?}", pt_entry);
                }
            }
        }

        log!("");
    }

    let mut rbp = regs.rbp as *const usize;
    let kernel_start_ptr = &kernel_start as *const u8 as *const usize;
    let kernel_end_ptr = &kernel_end as *const u8 as *const usize;
    while rbp >= kernel_start_ptr && rbp < kernel_end_ptr {
        let rip = unsafe { *rbp.offset(1) as *const u8 };
        log!("rbp = {:p}: return to rip = {:p}", rbp, rip);
        rbp = unsafe { *rbp as *const usize };
    }

    loop { }
}

test! {
    fn can_interrupt() {
        unsafe { cpu::int(48) }
    }
}
