use super::serial;
use libc::c_char;

extern "C" {
    static kernel_start: u8;
    static kernel_end: u8;
}

pub fn puts(s: &str) {
    serial::puts(s);
}

pub unsafe fn put_cstr(s: *const c_char) {
    serial::put_cstr(s);
}

pub unsafe fn print_stack_trace(mut frame: *const usize) {
    let kernel_start_ptr = &kernel_start as *const u8 as *const usize;
    let kernel_end_ptr = &kernel_end as *const u8 as *const usize;
    let user_start_ptr = 0x1000 as *const usize;
    let user_end_ptr = 0x1000_0000 as *const usize;
    while (frame >= kernel_start_ptr && frame < kernel_end_ptr) || (frame >= user_start_ptr && frame < user_end_ptr) {
        let pc = *frame.offset(1) as *const u8;
        log!("frame = {:p}: return to pc = {:p}", frame, pc);
        frame = *frame as *const usize;
    }
}
