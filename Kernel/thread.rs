use libc::{self,jmp_buf};
use std::boxed::{Box,FnBox};
use std::vec;
use super::arch::thread;

pub struct Thread {
    stack: vec::Vec<isize>,
    jmp_buf: jmp_buf
}

impl Thread {
    pub fn new(start: Box<FnBox()>) -> Thread {
        let mut stack = vec::from_elem(0, 512 * 512);
        let jmp_buf = thread::new_jmp_buf(start, &mut stack);
        Thread { stack: stack, jmp_buf: jmp_buf } 
    }

    pub fn jump(&self) -> ! {
        unsafe {
            libc::longjmp(&self.jmp_buf, 0);
        }
    }
}

