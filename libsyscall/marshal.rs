use core::slice::{self,SliceExt};
use core::str::{self,StrExt};

pub trait SyscallArgs {
    fn as_args(self) -> (usize, usize);
    fn from_args(arg1: usize, arg2: usize) -> Self;
}

pub trait SyscallResult {
    fn as_result(self) -> usize;
    fn from_result(value: usize) -> Self;
}

impl SyscallArgs for () {
    fn as_args(self) -> (usize, usize) {
        (0, 0)
    }

    fn from_args(_arg1: usize, _arg2: usize) {
    }
}

impl SyscallArgs for u32 {
    fn as_args(self) -> (usize, usize) {
        (self as usize, 0)
    }

    fn from_args(arg1: usize, _arg2: usize) -> u32 {
        arg1 as u32
    }
}

impl<'a, T> SyscallArgs for &'a [T] {
    fn as_args(self) -> (usize, usize) {
        (self.as_ptr() as usize, self.len())
    }

    fn from_args(arg1: usize, arg2: usize) -> &'a [T] {
        unsafe { slice::from_raw_parts(arg1 as *mut T, arg2) }
    }
}

impl<'a, T> SyscallArgs for &'a mut [T] {
    fn as_args(self) -> (usize, usize) {
        (self.as_mut_ptr() as usize, self.len())
    }

    fn from_args(arg1: usize, arg2: usize) -> &'a mut [T] {
        unsafe { slice::from_raw_parts_mut(arg1 as *mut T, arg2) }
    }
}

impl<'a> SyscallArgs for &'a str {
    fn as_args(self) -> (usize, usize) {
        self.as_bytes().as_args()
    }

    fn from_args(arg1: usize, arg2: usize) -> &'a str {
        str::from_utf8(SyscallArgs::from_args(arg1, arg2)).unwrap()
    }
}

impl SyscallResult for () {
    fn as_result(self) -> usize {
        0
    }

    fn from_result(_value: usize) {
    }
}

impl SyscallResult for usize {
    fn as_result(self) -> usize {
        self
    }

    fn from_result(value: usize) -> usize {
        value
    }
}
