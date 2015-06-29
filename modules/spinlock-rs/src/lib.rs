#![crate_name = "spin"]
#![crate_type = "lib"]
#![warn(missing_docs)]
#![feature(asm)]

//! Synchronization primitives based on spinning

#![cfg_attr(feature = "no_std", feature(no_std, core))]
#![cfg_attr(feature = "no_std", no_std)]

#[cfg(all(feature = "no_std", test))]
extern crate std;

#[cfg(feature = "no_std")]
#[macro_use]
extern crate core;

pub use mutex::*;
pub use rw_lock::*;

mod mutex;
mod rw_lock;

mod interrupts {
    #[inline]
    pub fn disable() -> usize {
        let rflags: usize;
        unsafe { asm!("pushfq ; cli ; pop $0" : "=r"(rflags)) };
        rflags & (1 << 9)
    }

    #[inline]
    pub fn restore(token: usize) {
        if token != 0 {
            unsafe { asm!("sti" :::: "volatile") };
        }
    }
}
