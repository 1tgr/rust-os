/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * arch/amd64/mod.rs
 * - Top-level file for amd64 architecture
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

pub use crate::arch::x86_common::*;

pub mod cpu;
pub mod isr;
pub mod mmu;
pub mod process;
pub mod thread;

#[inline]
pub fn disable_interrupts() -> usize {
    let rflags: usize;
    unsafe { asm!("pushfq ; cli ; pop $0" : "=r"(rflags)) };
    rflags & (1 << 9)
}

#[inline]
pub fn restore_interrupts(token: usize) {
    if token != 0 {
        unsafe { asm!("sti" :::: "volatile") };
    }
}

#[allow(unused_attributes)]
#[link_args = "-T arch/amd64/link.ld"]
#[link_args = "-L arch/amd64"]
extern "C" {}

#[link(name = ":setjmp.o")]
#[link(name = ":start.o")]
extern "C" {}
