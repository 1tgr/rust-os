use ::arch::cpu::{self,Regs};
use lazy_static::once::{self,Once};
use libc::jmp_buf;
use std::boxed::{self,FnBox};
use std::intrinsics;
use std::mem;

extern {
    static thread_entry_asm: u8;
    static syscall_entry_asm: u8;
}

type RegsHandler = Fn(&Regs) -> usize;

static mut SYSCALL_HANDLER: *mut Box<RegsHandler> = 0 as *mut _;

#[no_mangle]
pub unsafe fn syscall_entry(regs: &Regs) -> usize {
    let p = SYSCALL_HANDLER;
    if p == (0 as *mut _) {
        0
    } else {
        let b: &Box<RegsHandler> = &*p;
        b(regs)
    }
}

fn atomic_replace<T>(dest: &mut *mut T, src: *mut T) -> *mut T {
    let dest_ptr = dest as *mut *mut T as *mut usize;
    let src = src as usize;
    let old_dest: usize = unsafe { intrinsics::atomic_xchg(dest_ptr, src) };
    old_dest as *mut T
}

pub struct DropSyscall {
    old: *mut Box<RegsHandler>
}

impl Drop for DropSyscall {
    fn drop(&mut self) {
        let b: Box<Box<RegsHandler>> = unsafe {
            let p: *mut Box<RegsHandler> = atomic_replace(&mut SYSCALL_HANDLER, self.old);
            Box::from_raw(p)
        };

        mem::drop(b);
    }
}

pub fn set_syscall_handler(handler: Box<Fn(&Regs) -> usize + 'static>) -> DropSyscall {
    unsafe {
        let b1: Box<Box<RegsHandler>> = Box::new(handler);
        let p1: *mut Box<RegsHandler> = boxed::into_raw(b1);
        DropSyscall {
            old: atomic_replace(&mut SYSCALL_HANDLER, p1)
        }
    }
}

#[no_mangle]
pub fn thread_entry(p: *mut u8) -> ! {
    let start : Box<Box<FnBox()>>;
    unsafe {
        start = Box::from_raw(p as *mut _);
    }

    start();
    unreachable!()
}

pub fn new_jmp_buf<'a>(p: Box<FnBox() + 'a>, stack: &'static mut [u8]) -> jmp_buf {
    let pp = Box::new(p);
    let rsp : *mut u8 = unsafe { stack.as_mut_ptr().offset(stack.len() as isize - 8) };
    let rip : *const u8 = &thread_entry_asm;
    let rbx : *const u8 = &*pp as *const _ as *const _;
    mem::forget(pp);

    jmp_buf {
        rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0,
        rsp: rsp as i64,
        rip: rip as i64,
        rbx: rbx as i64
    }
}

pub unsafe fn jmp_user_mode(rip: *const u8, rsp: *const u8) -> ! {
    static INIT: Once = once::ONCE_INIT;
    INIT.call_once(|| {
        cpu::wrmsr(cpu::IA32_STAR, 0x00100008_00000000);
        cpu::wrmsr(cpu::IA32_LSTAR, &syscall_entry_asm as *const u8 as u64);
    });

    log!("jmp_user_mode({:p}, {:p})", rip, rsp);
    cpu::sysret(rip, rsp, 1 << 9)
}
