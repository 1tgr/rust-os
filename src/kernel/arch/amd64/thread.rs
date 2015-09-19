use alloc::boxed::FnBox;
use arch::cpu::{self,Regs};
use core::mem;
use libc::jmp_buf;
use once::{self,Once};
use prelude::*;
use singleton::{DropSingleton,Singleton};
use syscall::kernel::{Dispatch,PackedArgs};
use syscall::{Dispatcher,Handler};

extern {
    static thread_entry_asm: u8;
    static syscall_entry_asm: u8;
}

pub type RegsHandler = Fn(&Regs) -> usize;

lazy_static! {
    static ref SYSCALL_DISPATCH: Singleton<Box<Dispatch+'static>> = Singleton::<Box<Dispatch+'static>>::new();
}

#[no_mangle]
pub unsafe fn syscall_entry(regs: &Regs) -> isize {
    if let Some(ref d) = SYSCALL_DISPATCH.get() {
        let d: &Box<Dispatch> = d;
        let d: &Dispatch = &**d;
        d.dispatch(regs.rax as usize, PackedArgs::from_tuple((regs.rdi as usize, regs.rsi as usize, regs.rdx as usize, regs.rcx as usize, regs.r8 as usize, regs.r9 as usize)))
    } else {
        0
    }
}

pub type DropSyscallHandler<'a> = DropSingleton<'a, Box<Dispatch>>;

pub fn register_syscall_handler<'a, T>(handler: T) -> DropSyscallHandler<'a> where T : Handler + 'a {
    let b: Box<Dispatcher<T>> = Box::new(Dispatcher::new(handler));
    let b: Box<Dispatch + 'a> = b;
    let b: Box<Dispatch + 'static> = unsafe { mem::transmute(b) };
    SYSCALL_DISPATCH.register(b)
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

pub fn new_jmp_buf<'a>(p: Box<FnBox() + 'a>, stack_ptr: *mut u8) -> jmp_buf {
    let pp = Box::new(p);
    let rsp : *mut u8 = stack_ptr;
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
        const KERNEL_CS: u16 = 0x08;  // KERNEL_SS = 0x10 (+8)
        const USER_CS_32: u16 = 0x23; // USER_SS = 0x2B (+8); USER_CS_64 = 0x33 (+16)
        const RFLAGS_IF: u64 = 1 << 9;
        cpu::wrmsr(cpu::IA32_STAR, (USER_CS_32 as u64) << 48 | (KERNEL_CS as u64) << 32);
        cpu::wrmsr(cpu::IA32_LSTAR, &syscall_entry_asm as *const u8 as u64);
        cpu::wrmsr(cpu::IA32_SFMASK, RFLAGS_IF);
    });

    log!("jmp_user_mode({:p}, {:p})", rip, rsp);
    cpu::sysret(rip, rsp, 1 << 9)
}
