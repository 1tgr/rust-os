#[cfg(not(feature = "kernel"))]
macro_rules! syscalls {
    (
        $(
            $(#[$attrs:meta])*
            fn $name:ident($($arg_name:ident: $arg_ty:ty),+) -> $result:ty => $num:expr
        ),+
    ) => {
        use $crate::marshal::ErrNum;

        #[allow(non_camel_case_types)]
        enum Num {
            $(
                $name = $num,
            )+
        }

        $(
            $(#[$attrs])*
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
            $(#[$attrs:meta])*
            fn $name:ident($($arg_name:ident: $arg_ty:ty),+) -> $result:ty => $num:expr
        ),+
    ) => {
        use $crate::kernel::Dispatch;
        use $crate::marshal::{ErrNum,PackedArgs,SyscallArgs,SyscallResult};
        use core::fmt::Write;

        pub trait Handler {
            $(
                fn $name<'a>(&self, $($arg_name: $arg_ty),+) -> Result<$result, ErrNum>;
            )+
        }

        pub struct Dispatcher<W, T> {
            writer: W,
            handler: T
        }

        impl<W, T> Dispatcher<W, T> {
            pub fn new(writer: W, handler: T) -> Dispatcher<W, T> {
                Dispatcher {
                    writer: writer,
                    handler: handler
                }
            }
        }

        impl<T: Handler, W: Clone + Write> Dispatch for Dispatcher<W, T> {
            fn dispatch(&self, num: usize, mut args: PackedArgs) -> isize {
                match num {
                    $(
                        $num =>
                            (match SyscallArgs::from_args(&mut args) {
                                Ok(tuple) => {
                                    let mut writer = self.writer.clone();
                                    let _ = write!(writer, concat!(stringify!($name), "{:?}"), tuple);
                                    let ($($arg_name,)+) = tuple;
                                    let result = self.handler.$name($($arg_name),+);
                                    let _ = writeln!(writer, " => {:?}", result);
                                    result
                                },
                                Err(num) => Err(num)
                            }).as_result(),
                    )+
                    _ => 0
                }
            }
        }
    }
}
