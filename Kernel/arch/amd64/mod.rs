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

#[path = "../x86_common/mod.rs"]
mod x86_common;

pub use self::x86_common::debug;

pub mod thread;

pub fn init() {
    x86_common::vga::init();
}
