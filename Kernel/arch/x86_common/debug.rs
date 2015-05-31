use libc::c_char;
use super::serial;
use super::vga;

pub fn puts(s: &str) {
    serial::puts(s);
    vga::puts(s);
}

pub unsafe fn put_cstr(s: *const c_char) {
    serial::put_cstr(s);
    vga::put_cstr(s);
}
