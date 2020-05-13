use crate::ErrNum;
use core::convert::TryFrom;
use core::mem;
use core::slice;
use core::str::{self, Utf8Error};

pub struct PackedArgs {
    tuple: (usize, usize, usize, usize, usize, usize),
    len: u8,
}

impl Default for PackedArgs {
    fn default() -> Self {
        Self {
            tuple: (0, 0, 0, 0, 0, 0),
            len: 0,
        }
    }
}

impl PackedArgs {
    #[cfg(feature = "kernel")]
    pub fn new(arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize, arg6: usize) -> Self {
        Self {
            tuple: (arg1, arg2, arg3, arg4, arg5, arg6),
            len: 6,
        }
    }

    pub fn pop_front(&mut self) -> usize {
        match self.len {
            6 => {
                self.len = 5;
                self.tuple.0
            }
            5 => {
                self.len = 4;
                self.tuple.1
            }
            4 => {
                self.len = 3;
                self.tuple.2
            }
            3 => {
                self.len = 2;
                self.tuple.3
            }
            2 => {
                self.len = 1;
                self.tuple.4
            }
            1 => {
                self.len = 0;
                self.tuple.5
            }
            _ => panic!(),
        }
    }

    pub fn push_back(&mut self, item: usize) {
        match self.len {
            0 => {
                self.tuple.0 = item;
                self.len = 1;
            }
            1 => {
                self.tuple.1 = item;
                self.len = 2;
            }
            2 => {
                self.tuple.2 = item;
                self.len = 3;
            }
            3 => {
                self.tuple.3 = item;
                self.len = 4;
            }
            4 => {
                self.tuple.4 = item;
                self.len = 5;
            }
            5 => {
                self.tuple.5 = item;
                self.len = 6;
            }
            _ => panic!("tuple is full"),
        }
    }

    #[cfg(all(target_arch = "x86_64", not(feature = "kernel")))]
    pub unsafe fn syscall(self, num: u32) -> isize {
        let result: isize;
        match self.len {
            0 => {
                asm!("syscall"
                : "={rax}"(result)
                : "{rax}"(num)
                : "rcx", "r11", "cc",      // syscall/sysret clobbers rcx, r11, rflags
                    "memory"
                : "volatile");
            }
            1 => {
                asm!("syscall"
                : "={rax}"(result)
                : "{rax}"(num), "{rdi}"(self.tuple.0)
                : "rcx", "r11", "cc", "memory"
                : "volatile");
            }
            2 => {
                asm!("syscall"
                : "={rax}"(result)
                : "{rax}"(num), "{rdi}"(self.tuple.0), "{rsi}"(self.tuple.1)
                : "rcx", "r11", "cc", "memory"
                : "volatile");
            }
            3 => {
                asm!("syscall"
                : "={rax}"(result)
                : "{rax}"(num), "{rdi}"(self.tuple.0), "{rsi}"(self.tuple.1), "{rdx}"(self.tuple.2)
                : "rcx", "r11", "cc", "memory"
                : "volatile");
            }
            4 => {
                asm!("syscall"
                : "={rax}"(result)
                : "{rax}"(num), "{rdi}"(self.tuple.0), "{rsi}"(self.tuple.1), "{rdx}"(self.tuple.2), "{r8}"(self.tuple.3)
                : "rcx", "r11", "cc", "memory"
                : "volatile");
            }
            5 => {
                asm!("syscall"
                : "={rax}"(result)
                : "{rax}"(num), "{rdi}"(self.tuple.0), "{rsi}"(self.tuple.1), "{rdx}"(self.tuple.2), "{r8}"(self.tuple.3), "{r9}"(self.tuple.4)
                : "rcx", "r11", "cc", "memory"
                : "volatile");
            }
            _ => {
                asm!("syscall"
                : "={rax}"(result)
                : "{rax}"(num), "{rdi}"(self.tuple.0), "{rsi}"(self.tuple.1), "{rdx}"(self.tuple.2), "{r8}"(self.tuple.3), "{r9}"(self.tuple.4), "{r10}"(self.tuple.5)
                : "rcx", "r11", "cc", "memory"
                : "volatile");
            }
        }
        result
    }
}

pub trait SyscallArgs: Sized {
    type Parsed;
    fn as_args(self, args: &mut PackedArgs);
    fn from_args(args: &mut PackedArgs) -> Self::Parsed;

    fn into_args(self) -> PackedArgs {
        let mut args = PackedArgs::default();
        self.as_args(&mut args);
        args
    }
}

pub trait SyscallResult {
    fn as_result(self) -> isize;
    fn from_result(value: isize) -> Self;
}

impl SyscallArgs for () {
    type Parsed = Self;

    fn as_args(self, _args: &mut PackedArgs) {}

    fn from_args(_args: &mut PackedArgs) -> Self {}
}

impl SyscallArgs for bool {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(if self { 1 } else { 0 })
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front() != 0
    }
}

impl SyscallArgs for u8 {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front() as Self
    }
}

impl SyscallArgs for u16 {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front() as Self
    }
}

impl SyscallArgs for u32 {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front() as Self
    }
}

impl SyscallArgs for i8 {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front() as Self
    }
}

impl SyscallArgs for i16 {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front() as Self
    }
}

impl SyscallArgs for i32 {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front() as Self
    }
}

impl SyscallArgs for usize {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self);
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front()
    }
}

impl<T> SyscallArgs for *const T {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize)
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front() as Self
    }
}

impl<T> SyscallArgs for *mut T {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize)
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        args.pop_front() as Self
    }
}

impl<'a, T> SyscallArgs for &'a [T] {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        (self.as_ptr(), self.len()).as_args(args);
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        let (ptr, len) = <(*const T, usize) as SyscallArgs>::from_args(args);
        unsafe { slice::from_raw_parts(ptr, len) }
    }
}

impl<'a, T> SyscallArgs for &'a mut [T] {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        (self.as_mut_ptr(), self.len()).as_args(args)
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        let (ptr, len) = <(*mut T, usize) as SyscallArgs>::from_args(args);
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }
}

impl<'a> SyscallArgs for &'a str {
    type Parsed = Result<Self, Utf8Error>;

    fn as_args(self, args: &mut PackedArgs) {
        self.as_bytes().as_args(args)
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self, Utf8Error> {
        let bytes = <&[u8] as SyscallArgs>::from_args(args);
        str::from_utf8(bytes)
    }
}

impl<T> SyscallArgs for extern "C" fn(T) {
    type Parsed = Self;

    fn as_args(self, args: &mut PackedArgs) {
        (self as usize).as_args(args)
    }

    fn from_args(args: &mut PackedArgs) -> Self {
        let ptr = usize::from_args(args);
        unsafe { mem::transmute::<usize, Self>(ptr) }
    }
}

impl<T: SyscallArgs> SyscallArgs for (T,) {
    type Parsed = (T::Parsed,);

    fn as_args(self, args: &mut PackedArgs) {
        self.0.as_args(args)
    }

    fn from_args(args: &mut PackedArgs) -> Self::Parsed {
        (T::from_args(args),)
    }
}

impl<T1: SyscallArgs, T2: SyscallArgs> SyscallArgs for (T1, T2) {
    type Parsed = (T1::Parsed, T2::Parsed);

    fn as_args(self, args: &mut PackedArgs) {
        self.0.as_args(args);
        self.1.as_args(args);
    }

    fn from_args(args: &mut PackedArgs) -> Self::Parsed {
        let a = T1::from_args(args);
        let b = T2::from_args(args);
        (a, b)
    }
}

impl<T1: SyscallArgs, T2: SyscallArgs, T3: SyscallArgs> SyscallArgs for (T1, T2, T3) {
    type Parsed = (T1::Parsed, T2::Parsed, T3::Parsed);

    fn as_args(self, args: &mut PackedArgs) {
        self.0.as_args(args);
        self.1.as_args(args);
        self.2.as_args(args);
    }

    fn from_args(args: &mut PackedArgs) -> Self::Parsed {
        let a = T1::from_args(args);
        let b = T2::from_args(args);
        let c = T3::from_args(args);
        (a, b, c)
    }
}

impl<T: SyscallResult> SyscallResult for Result<T, ErrNum> {
    fn as_result(self) -> isize {
        match self {
            Ok(x) => x.as_result(),
            Err(num) => {
                let num: usize = num.into();
                -(num as isize)
            }
        }
    }

    fn from_result(value: isize) -> Self {
        if value < 0 {
            Err(ErrNum::try_from(-value as usize).unwrap_or(ErrNum::NotSupported))
        } else {
            Ok(SyscallResult::from_result(value))
        }
    }
}

impl SyscallResult for ! {
    fn as_result(self) -> isize {
        0
    }

    fn from_result(_value: isize) -> Self {
        unreachable!()
    }
}

impl SyscallResult for () {
    fn as_result(self) -> isize {
        0
    }

    fn from_result(_value: isize) {}
}

impl SyscallResult for bool {
    fn as_result(self) -> isize {
        if self {
            1
        } else {
            0
        }
    }

    fn from_result(value: isize) -> Self {
        value != 0
    }
}

impl SyscallResult for i32 {
    fn as_result(self) -> isize {
        self as isize
    }

    fn from_result(value: isize) -> Self {
        value as Self
    }
}

impl SyscallResult for u32 {
    fn as_result(self) -> isize {
        self as isize
    }

    fn from_result(value: isize) -> Self {
        value as Self
    }
}

impl SyscallResult for usize {
    fn as_result(self) -> isize {
        self as isize
    }

    fn from_result(value: isize) -> Self {
        value as Self
    }
}

impl<'a, T> SyscallResult for *const T {
    fn as_result(self) -> isize {
        self as isize
    }

    fn from_result(value: isize) -> Self {
        value as Self
    }
}

impl<'a, T> SyscallResult for *mut T {
    fn as_result(self) -> isize {
        self as isize
    }

    fn from_result(value: isize) -> Self {
        value as Self
    }
}
