#![crate_name = "syscall"]

#![feature(asm)]
#![feature(core)]
#![feature(no_std)]
#![no_std]

#[macro_use] extern crate core;

mod marshal;

#[cfg(not(feature = "kernel"))]
mod user;

#[cfg(feature = "kernel")]
pub mod kernel;

#[cfg(not(feature = "kernel"))]
macro_rules! syscalls {
    (
        $(
            $num:expr => $name:ident($arg:ty) -> $result:ty
        ),+
    ) => {
        use core::result::Result;
        use $crate::marshal::ErrNum;

        #[allow(non_camel_case_types)]
        enum Num {
            $(
                $name = $num,
            )+
        }

        $(
            pub fn $name<'a>(arg: $arg) -> Result<$result, ErrNum> {
                unsafe { $crate::user::syscall(Num::$name as u32, arg) }
            }
        )+
    }
}

#[cfg(feature = "kernel")]
macro_rules! syscalls {
    (
        $(
            $num:expr => $name:ident($arg:ty) -> $result:ty
        ),+
    ) => {
        use core::result::Result::{self,Ok,Err};
        use $crate::marshal::{ErrNum,SyscallArgs,SyscallResult};
        use $crate::kernel::Dispatch;

        pub trait Handler {
            $(
                fn $name<'a>(&self, arg: $arg) -> Result<$result, ErrNum>;
            )+
        }

        pub struct Dispatcher<T> {
            handler: T
        }

        impl<T> Dispatcher<T> {
            pub fn new(handler: T) -> Dispatcher<T> {
                Dispatcher {
                    handler: handler
                }
            }
        }

        impl<T> Dispatch for Dispatcher<T> where T : Handler {
            fn dispatch(&self, num: usize, arg1: usize, arg2: usize) -> isize {
                match num {
                    $(
                        $num =>
                            (match SyscallArgs::from_args(arg1, arg2) {
                                Ok(args) => self.handler.$name(args),
                                Err(num) => Err(num) 
                            }).as_result(),
                    )+
                    _ => 0
                }
            }
        }
    }
}

mod table;

pub use marshal::ErrNum;
pub use table::*;
