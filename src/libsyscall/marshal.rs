use core::mem;
use core::slice;
use core::str::{self,StrExt};
use super::{ErrNum,Result};

pub struct TupleDeque6<T> {
    tuple: (T, T, T, T, T, T),
    len: u8
}

impl<T: Default> TupleDeque6<T> {
    pub fn new() -> Self {
        TupleDeque6 {
            tuple: <_>::default(),
            len: 0
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        match self.len {
            6 => { self.len = 5; Some(mem::replace(&mut self.tuple.0, T::default())) },
            5 => { self.len = 4; Some(mem::replace(&mut self.tuple.1, T::default())) },
            4 => { self.len = 3; Some(mem::replace(&mut self.tuple.2, T::default())) },
            3 => { self.len = 2; Some(mem::replace(&mut self.tuple.3, T::default())) },
            2 => { self.len = 1; Some(mem::replace(&mut self.tuple.4, T::default())) },
            1 => { self.len = 0; Some(mem::replace(&mut self.tuple.5, T::default())) },
            _ => None
        }
    }
}

impl<T> TupleDeque6<T> {
    #[allow(dead_code)]
    pub fn from_tuple(tuple: (T, T, T, T, T, T)) -> Self {
        TupleDeque6 {
            tuple: tuple,
            len: 6
        }
    }

    pub fn push_back(&mut self, item: T) {
        match self.len {
            0 => { self.tuple.0 = item; self.len = 1; },
            1 => { self.tuple.1 = item; self.len = 2; },
            2 => { self.tuple.2 = item; self.len = 3; },
            3 => { self.tuple.3 = item; self.len = 4; },
            4 => { self.tuple.4 = item; self.len = 5; },
            5 => { self.tuple.5 = item; self.len = 6; },
            _ => { panic!("tuple is full") }
        }
    }

    pub fn unwrap(self) -> ((T, T, T, T, T, T), u8) {
        (self.tuple, self.len)
    }
}

pub type PackedArgs = TupleDeque6<usize>;

pub trait SyscallArgs : Sized {
    fn as_args(self, args: &mut PackedArgs);
    fn from_args(args: &mut PackedArgs) -> Result<Self>;
}

pub trait SyscallResult {
    fn as_result(self) -> isize;
    fn from_result(value: isize) -> Self;
}

impl SyscallArgs for () {
    fn as_args(self, _args: &mut PackedArgs) {
    }

    fn from_args(_args: &mut PackedArgs) -> Result<Self> {
        Ok(())
    }
}

impl SyscallArgs for bool {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(if self { 1 } else { 0 })
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap() != 0)
    }
}

impl SyscallArgs for u8 {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap() as Self)
    }
}

impl SyscallArgs for u16 {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap() as Self)
    }
}

impl SyscallArgs for u32 {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap() as Self)
    }
}

impl SyscallArgs for i8 {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap() as Self)
    }
}

impl SyscallArgs for i16 {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap() as Self)
    }
}

impl SyscallArgs for i32 {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap() as Self)
    }
}

impl SyscallArgs for usize {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap())
    }
}

impl<T> SyscallArgs for *const T {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize)
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap() as Self)
    }
}

impl<T> SyscallArgs for *mut T {
    fn as_args(self, args: &mut PackedArgs) {
        args.push_back(self as usize)
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok(args.pop_front().unwrap() as Self)
    }
}

impl<'a, T> SyscallArgs for &'a [T] {
    fn as_args(self, args: &mut PackedArgs) {
        (self.as_ptr(), self.len()).as_args(args);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        let (ptr, len) = try!(SyscallArgs::from_args(args));
        Ok(unsafe { slice::from_raw_parts(ptr, len) })
    }
}

impl<'a, T> SyscallArgs for &'a mut [T] {
    fn as_args(self, args: &mut PackedArgs) {
        (self.as_mut_ptr(), self.len()).as_args(args)
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        let (ptr, len) = try!(SyscallArgs::from_args(args));
        Ok(unsafe { slice::from_raw_parts_mut(ptr, len) })
    }
}

impl<'a> SyscallArgs for &'a str {
    fn as_args(self, args: &mut PackedArgs) {
        self.as_bytes().as_args(args)
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        let bytes = try!(SyscallArgs::from_args(args));
        match str::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_) => Err(ErrNum::Utf8Error)
        }
    }
}

impl<T: SyscallArgs> SyscallArgs for (T,) {
    fn as_args(self, args: &mut PackedArgs) {
        self.0.as_args(args)
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        Ok((try!(SyscallArgs::from_args(args)),))
    }
}

impl<T1: SyscallArgs, T2: SyscallArgs> SyscallArgs for (T1, T2) {
    fn as_args(self, args: &mut PackedArgs) {
        self.0.as_args(args);
        self.1.as_args(args);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        let a = try!(SyscallArgs::from_args(args));
        let b = try!(SyscallArgs::from_args(args));
        Ok((a, b))
    }
}

impl<T1: SyscallArgs, T2: SyscallArgs, T3: SyscallArgs> SyscallArgs for (T1, T2, T3) {
    fn as_args(self, args: &mut PackedArgs) {
        self.0.as_args(args);
        self.1.as_args(args);
        self.2.as_args(args);
    }

    fn from_args(args: &mut PackedArgs) -> Result<Self> {
        let a = try!(SyscallArgs::from_args(args));
        let b = try!(SyscallArgs::from_args(args));
        let c = try!(SyscallArgs::from_args(args));
        Ok((a, b, c))
    }
}

impl<T: SyscallResult> SyscallResult for Result<T> {
    fn as_result(self) -> isize {
        match self {
            Ok(x) => x.as_result(),
            Err(num) => -(num as isize)
        }
    }

    fn from_result(value: isize) -> Result<T> {
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
