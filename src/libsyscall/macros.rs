#[cfg(not(feature = "kernel"))]
macro_rules! syscalls {
    (
        $(
            $num:expr => $name:ident($($arg_name:ident: $arg_ty:ty),+) -> $result:ty
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
            pub fn $name<'a>($($arg_name: $arg_ty),+) -> Result<$result, ErrNum> {
                unsafe { $crate::user::syscall(Num::$name as u32, ($($arg_name,)+)) }
            }
        )+
    }
}

#[cfg(feature = "kernel")]
macro_rules! syscalls {
    (
        $(
            $num:expr => $name:ident($($arg_name:ident: $arg_ty:ty),+) -> $result:ty
        ),+
    ) => {
        use $crate::kernel::Dispatch;
        use $crate::marshal::{ErrNum,PackedArgs,SyscallArgs,SyscallResult};
        use core::result::Result::{self,Ok,Err};

        pub trait Handler {
            $(
                fn $name<'a>(&self, $($arg_name: $arg_ty),+) -> Result<$result, ErrNum>;
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
            fn dispatch(&self, num: usize, mut args: PackedArgs) -> isize {
                match num {
                    $(
                        $num =>
                            (match SyscallArgs::from_args(&mut args) {
                                Ok(($($arg_name,)+)) => self.handler.$name($($arg_name),+),
                                Err(num) => Err(num)
                            }).as_result(),
                    )+
                    _ => 0
                }
            }
        }
    }
}
