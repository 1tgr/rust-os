use libc::{c_void,jmp_buf};
use std::boxed::{Box,FnBox};
use std::mem;

extern {
    static thread_entry_asm: c_void;
}

#[no_mangle]
pub fn thread_entry(p: *mut c_void) -> ! {
    let start : Box<Box<FnBox()>>;
    unsafe {
        start = Box::from_raw(p as *mut _);
    }

    start();
    loop { }
}

pub fn new_jmp_buf<'a>(p: Box<FnBox() + 'a>, stack: *mut u8, stack_len: usize) -> jmp_buf {
    let pp = Box::new(p);
    let rsp : *mut u8 = unsafe { stack.offset(stack_len as isize - 8) };
    let rip : *const c_void = &thread_entry_asm;
    let rbx : *const c_void = &*pp as *const _ as *mut _;
    mem::forget(pp);

    jmp_buf {
        rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0,
        rsp: rsp as i64,
        rip: rip as i64,
        rbx: rbx as i64
    }
}

