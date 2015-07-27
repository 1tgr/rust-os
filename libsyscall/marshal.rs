use core::mem;
use core::result::Result::{self,Ok,Err};
use core::slice::{self,SliceExt};
use core::str::{self,StrExt};

#[repr(usize)]
pub enum ErrNum {
    Utf8Error = 1,
    OutOfMemory = 2
}

pub trait SyscallArgs {
    fn as_args(self) -> (usize, usize);
    fn from_args(arg1: usize, arg2: usize) -> Result<Self, ErrNum>;
}

pub trait SyscallResult {
    fn as_result(self) -> isize;
    fn from_result(value: isize) -> Self;
}

impl SyscallArgs for () {
    fn as_args(self) -> (usize, usize) {
        (0, 0)
    }

    fn from_args(_arg1: usize, _arg2: usize) -> Result<Self, ErrNum >{
        Ok(())
    }
}

impl SyscallArgs for i32 {
    fn as_args(self) -> (usize, usize) {
        (self as usize, 0)
    }

    fn from_args(arg1: usize, _arg2: usize) -> Result<Self, ErrNum> {
        Ok(arg1 as Self)
    }
}

impl SyscallArgs for usize {
    fn as_args(self) -> (usize, usize) {
        (self, 0)
    }

    fn from_args(arg1: usize, _arg2: usize) -> Result<Self, ErrNum> {
        Ok(arg1)
    }
}

impl<T> SyscallArgs for *const T {
    fn as_args(self) -> (usize, usize) {
        (self as usize, 0)
    }

    fn from_args(arg1: usize, _arg2: usize) -> Result<Self, ErrNum> {
        Ok(arg1 as Self)
    }
}

impl<T> SyscallArgs for *mut T {
    fn as_args(self) -> (usize, usize) {
        (self as usize, 0)
    }

    fn from_args(arg1: usize, _arg2: usize) -> Result<Self, ErrNum> {
        Ok(arg1 as Self)
    }
}

impl<'a, T> SyscallArgs for &'a [T] {
    fn as_args(self) -> (usize, usize) {
        (self.as_ptr() as usize, self.len())
    }

    fn from_args(arg1: usize, arg2: usize) -> Result<Self, ErrNum> {
        Ok(unsafe { slice::from_raw_parts(arg1 as *mut T, arg2) })
    }
}

impl<'a, T> SyscallArgs for &'a mut [T] {
    fn as_args(self) -> (usize, usize) {
        (self.as_mut_ptr() as usize, self.len())
    }

    fn from_args(arg1: usize, arg2: usize) -> Result<Self, ErrNum> {
        Ok(unsafe { slice::from_raw_parts_mut(arg1 as *mut T, arg2) })
    }
}

impl<'a> SyscallArgs for &'a str {
    fn as_args(self) -> (usize, usize) {
        self.as_bytes().as_args()
    }

    fn from_args(arg1: usize, arg2: usize) -> Result<Self, ErrNum> {
        match str::from_utf8(try!(SyscallArgs::from_args(arg1, arg2))) {
            Ok(s) => Ok(s),
            Err(_) => Err(ErrNum::Utf8Error)
        }
    }
}

impl<T: SyscallResult> SyscallResult for Result<T, ErrNum> {
    fn as_result(self) -> isize {
        match self {
            Ok(x) => x.as_result(),
            Err(num) => -(num as isize)
        }
    }

    fn from_result(value: isize) -> Result<T, ErrNum> {
        if value < 0 {
            Err(unsafe { mem::transmute(-value) })
        } else {
            Ok(SyscallResult::from_result(value))
        }
    }
}

impl SyscallResult for () {
    fn as_result(self) -> isize {
        0
    }

    fn from_result(_value: isize) {
    }
}

impl SyscallResult for bool {
    fn as_result(self) -> isize {
        if self { 1 } else { 0 }
    }

    fn from_result(value: isize) -> Self {
        value != 0
    }
}

impl SyscallResult for usize {
    fn as_result(self) -> isize {
        self as isize
    }

    fn from_result(value: isize) -> Self {
        value as usize
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
