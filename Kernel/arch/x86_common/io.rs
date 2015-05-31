/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * arch/x86/x86_io.rs
 * - Support for the x86 IO bus
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

/// Write a byte to the specified port
pub unsafe fn outb(port: u16, val: u8) {
    asm!("outb %al, %dx" : : "{dx}" (port), "{al}" (val) : : "volatile");
}

/// Read a single byte from the specified port
pub unsafe fn inb(port: u16) -> u8 {
	let ret : u8;
	asm!("inb %dx, %al" : "={al}" (ret) : "{dx}" (port) : : "volatile");
	return ret;
}

