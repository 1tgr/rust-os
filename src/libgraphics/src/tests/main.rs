#![feature(link_args)]
#![feature(start)]

extern crate alloc;
extern crate alloc_system;
extern crate core;
extern crate graphics;
extern crate rt;

#[cfg(target_arch = "x86_64")]
#[allow(unused_attributes)]
#[link_args = "-T libsyscall/arch/amd64/link.ld"]
extern "C" {}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    for &(fixture_name, fixture) in graphics::TEST_FIXTURES {
        for &(test_name, test_fn) in fixture {
            println!("begin {}::{}", fixture_name, test_name);
            test_fn();
            println!("end {}::{}\n", fixture_name, test_name);
        }
    }

    0
}
