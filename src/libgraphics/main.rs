#![feature(link_args)]
#![feature(start)]

extern crate graphics;

#[cfg(target_arch="x86_64")]
#[link_args = "-T ../libsyscall/arch/amd64/link.ld"]
extern {
}

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
