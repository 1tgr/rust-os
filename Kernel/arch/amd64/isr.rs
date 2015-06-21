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
    error: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64
}

fn read_cr2() -> *const u8 {
    let cr2: *const u8;
    unsafe { asm!("mov %cr2, $0" : "=r"(cr2)) };
    cr2
}

#[no_mangle]
pub unsafe extern fn isr(regs: &Regs) {
    match regs.error {
        0 => log!("interrupt 0x{:x}", regs.num),
        1 => log!("irq {}", regs.num),
        _ => {
            let cr2 = read_cr2();
            log!("exception: error={:?}  cr2={:-16p}", regs.error, cr2);
            if regs.num == 14 {
                log!("page fault: {} {} in {} mode",
                     if (regs.error & 1) != 0 { "protection violation" } else { "page not present" },
                     if (regs.error & 2) != 0 { "writing" } else { "reading" },
                     if (regs.error & 4) != 0 { "user" } else { "kernel" });
            }

            log!("ss:rsp={:x}:{:-16x}  cs:rip={:x}:{:-16x} rflags={:-16x}", regs.ss, regs.rsp, regs.cs, regs.rip, regs.rflags);
            log!("rax={:-16x} rbx={:-16x} rcx={:-16x} rdx={:-16x}", regs.rax, regs.rbx, regs.rcx, regs.rdx);
            log!("rbp={:-16x} rdi={:-16x} rsi={:-16x}", regs.rbp, regs.rdi, regs.rsi);
            log!(" r8={:-16x}  r9={:-16x} r10={:-16x} r11={:-16x}", regs.r8, regs.r9, regs.r10, regs.r11);
            log!("r12={:-16x} r12={:-16x} r14={:-16x} r15={:-16x}", regs.r12, regs.r13, regs.r14, regs.r15);
            loop { }
        }
    };
}

test! {
    fn can_interrupt() {
        unsafe {
            asm!("int $0" :: "N"(0x30) :: "memory");
        }
    }
}
