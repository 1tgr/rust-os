/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * arch/x86/debug.rs
 * - Debug output channel
 *
 * Writes debug to the standard PC serial port (0x3F8 .. 0x3FF)
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */
use arch::cpu;
use libc::c_char;
use spin::{StaticMutex,STATIC_MUTEX_INIT};

static MUTEX: StaticMutex = STATIC_MUTEX_INIT;

unsafe fn putb(b: u8) {
    // Wait for the serial port's fifo to not be empty
    while (cpu::inb(0x3F8+5) & 0x20) == 0 {
        // Do nothing
    }

    // Send the byte out the serial port
    cpu::outb(0x3F8, b);

    // Also send to the bochs 0xe9 hack
    cpu::outb(0xe9, b);
}

pub fn puts(s: &str) {
    let _x = lock!(MUTEX);
	for b in s.bytes() {
		unsafe { putb(b) };
	}
}

pub unsafe fn put_cstr(s: *const c_char) {
    let _x = lock!(MUTEX);
    let mut s = s;
    while *s != 0 {
        putb(*s as u8);
        s = s.offset(1);
    }
}
