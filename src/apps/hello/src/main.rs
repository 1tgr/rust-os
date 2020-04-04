#![feature(link_args)]
#![feature(start)]

extern crate alloc_system;
extern crate rt;

#[cfg(target_arch = "x86_64")]
#[allow(unused_attributes)]
#[link_args = "-T ../../libsyscall/arch/amd64/link.ld"]
extern "C" {}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    println!("hello world");
    0
}
