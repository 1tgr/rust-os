use core::ptr;

pub fn wait_for_interrupt() {
    unsafe { asm!("wfe") }
}

pub fn current_frame() -> *const usize {
    ptr::null()
}
