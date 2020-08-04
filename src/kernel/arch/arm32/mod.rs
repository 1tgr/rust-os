pub mod cpu;
pub mod debug;
pub mod phys_mem;

#[inline]
pub fn disable_interrupts() -> usize {
    0
}

#[inline]
pub fn restore_interrupts(_token: usize) {}

#[allow(unused_attributes)]
#[link_args = "-T arch/arm32/link.ld"]
#[link_args = "-L arch/arm32"]
extern "C" {}

#[link(name = ":setjmp.o")]
#[link(name = ":start.o")]
extern "C" {}
