use libc::c_char;

const UART_DR: u32 = 0x3F201000;
const UART_FR: u32 = 0x3F201018;

unsafe fn mmio_write(reg: u32, val: u32) {
    (reg as *mut u32).write_volatile(val)
}

unsafe fn mmio_read(reg: u32) -> u32 {
    (reg as *const u32).read_volatile()
}

fn transmit_fifo_full() -> bool {
    unsafe { mmio_read(UART_FR) & (1 << 5) > 0 }
}

fn receive_fifo_empty() -> bool {
    unsafe { mmio_read(UART_FR) & (1 << 4) > 0 }
}

fn writec(c: u8) {
    while transmit_fifo_full() {}
    unsafe { mmio_write(UART_DR, c as u32) }
}

pub fn puts(s: &str) {
    for c in s.bytes() {
        writec(c);
    }
}

pub unsafe fn put_cstr(mut s: *const c_char) {
    while *s != 0 {
        writec(*s as u8);
        s = s.offset(1);
    }
}

pub unsafe fn print_stack_trace(_frame: *const usize) {}
