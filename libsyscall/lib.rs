#![crate_name = "syscall"]

#![feature(asm)]
#![feature(core)]
#![feature(no_std)]
#![no_std]

#[macro_use] extern crate core;

pub mod marshal;

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
        #[allow(non_camel_case_types)]
        enum Num {
            $(
                $name = $num,
            )+
        }

        $(
            pub fn $name<'a>(arg: $arg) -> $result {
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
        use $crate::marshal::{SyscallArgs,SyscallResult};
        use $crate::kernel::Dispatch;

        pub trait Handler {
            $(
                fn $name<'a>(&self, arg: $arg) -> $result;
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
            fn dispatch(&self, num: usize, arg1: usize, arg2: usize) -> usize {
                match num {
                    $(
                        $num => self.handler.$name(SyscallArgs::from_args(arg1, arg2)).as_result(),
                    )+
                    _ => 0
                }
            }
        }
    }
}

mod table;

pub use table::*;
