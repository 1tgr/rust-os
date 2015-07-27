use ::marshal::{SyscallArgs,SyscallResult};

pub unsafe fn syscall<T: SyscallArgs, U: SyscallResult>(num: u32, arg: T) -> U {
    let (arg1, arg2) = arg.as_args();
    let result: usize;
    asm!("syscall"
         : "={rax}"(result)
         : "{rax}"(num), "{rdi}"(arg1), "{rsi}"(arg2)
         : "{rcx}", "{r11}", "cc",      // syscall/sysret clobbers rcx, r11, rflags
           "memory" 
         : "volatile");
    SyscallResult::from_result(result)
}
