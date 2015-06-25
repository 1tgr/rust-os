use ::arch::cpu;
use ::arch::mmu;
use ::arch::x86_common::io;

#[repr(C)]
#[derive(Debug)]
pub struct Regs {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    num: u64,
    error: i64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64
}

extern {
    static kernel_start: u8;
    static kernel_end: u8;
}

fn interrupt(regs: &Regs) {
    log!("interrupt {}", regs.num);
}

fn irq(regs: &Regs) {
    unsafe {
        if regs.num >= 8 {
            io::outb(0xa0, 0x20);
        }
     
        io::outb(0x20, 0x20);
    }
}

fn exception(regs: &Regs) {
    let cr2: *const u8 = cpu::read_cr2();
    log!("exception {}: error=0x{:x}  cr2={:p}", regs.num, regs.error, cr2);
    log!("ss:rsp={:x}:{:-16x}  cs:rip={:x}:{:-16x} rflags={:x}", regs.ss, regs.rsp, regs.cs, regs.rip, regs.rflags);
    log!("rax={:-16x} rbx={:-16x} rcx={:-16x} rdx={:-16x}", regs.rax, regs.rbx, regs.rcx, regs.rdx);
    log!("rbp={:-16x} rdi={:-16x} rsi={:-16x}", regs.rbp, regs.rdi, regs.rsi);
    log!(" r8={:-16x}  r9={:-16x} r10={:-16x} r11={:-16x}", regs.r8, regs.r9, regs.r10, regs.r11);
    log!("r12={:-16x} r12={:-16x} r14={:-16x} r15={:-16x}", regs.r12, regs.r13, regs.r14, regs.r15);
    log!("");

    if regs.num == 14 {
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

#[no_mangle]
pub extern fn isr(regs: &Regs) {
    match regs.error {
        -1 => interrupt(regs),
        -2 => irq(regs),
        _ => exception(regs)
    };
}

test! {
    fn can_interrupt() {
        unsafe { cpu::int(48) }
    }
}
