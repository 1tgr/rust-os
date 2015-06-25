#![feature(asm)]
#![feature(lang_items)]

pub mod unwind;

unsafe fn syscall(num: u32, arg1: usize, arg2: usize) -> usize {
    let result;
    asm!("syscall"
         : "={rax}"(result)
         : "{rax}"(num), "{rdi}"(arg1), "{rsi}"(arg2)
         : "{rcx}", "{r11}", "cc",      // syscall/sysret clobbers rcx, r11, rflags
           "memory" 
         : "volatile");
    result
}

#[no_mangle]
#[link_section=".init"]
pub unsafe extern fn start() {
    let s = "hello world";
    syscall(0, s.as_ptr() as usize, s.len());
    syscall(1, 0x1234, 0);
}
