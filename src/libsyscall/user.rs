use ::marshal::{ErrNum,SyscallArgs,SyscallResult};
use core::result::Result;

pub unsafe fn syscall<T: SyscallArgs, U: SyscallResult>(num: u32, arg: T) -> Result<U, ErrNum> {
    let (arg1, arg2) = arg.as_args();
    let result: isize;
    asm!("syscall"
         : "={rax}"(result)
         : "{rax}"(num), "{rdi}"(arg1), "{rsi}"(arg2)
         : "rcx", "r11", "cc",      // syscall/sysret clobbers rcx, r11, rflags
           "memory" 
         : "volatile");
    SyscallResult::from_result(result)
}
