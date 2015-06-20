#![feature(asm)]
#![feature(lang_items)]	//< unwind needs to define lang items

pub mod unwind;

#[no_mangle]
#[link_section=".init"]
pub unsafe extern fn start() {
    asm!("syscall");
    loop { }
}
