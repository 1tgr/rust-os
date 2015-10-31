use super::ErrNum;
use marshal::{PackedArgs,SyscallArgs,SyscallResult};

pub mod libc_helpers;
pub mod unwind;

pub unsafe fn syscall<T: SyscallArgs, U: SyscallResult>(num: u32, args: T) -> Result<U, ErrNum> {
    let mut args_vec = PackedArgs::new();
    args.as_args(&mut args_vec);

    let (args, _) = args_vec.unwrap();
    let result: isize;
    asm!("syscall"
         : "={rax}"(result)
         : "{rax}"(num), "{rdi}"(args.0), "{rsi}"(args.1), "{rdx}"(args.2), "{rcx}"(args.3), "{r8}"(args.4), "{r9}"(args.5)
         : "rcx", "r11", "cc",      // syscall/sysret clobbers rcx, r11, rflags
           "memory"
         : "volatile");
    SyscallResult::from_result(result)
}
