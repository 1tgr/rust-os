#[cfg(not(feature = "kernel"))]
macro_rules! syscalls {
    (
        $(
            $(#[$attrs:meta])*
            fn $name:ident($($arg_name:ident: $arg_ty:ty),*) -> $result:ty => $num:expr
        ),+
    ) => {
        use $crate::ErrNum;
        use crate::marshal::{SyscallArgs, SyscallResult};

        #[allow(non_camel_case_types)]
        enum Num {
            $(
                $name = $num,
            )+
        }

        $(
            $(#[$attrs])*
            pub fn $name<'a>($($arg_name: $arg_ty),*) -> Result<$result, ErrNum> {
                let args = SyscallArgs::into_args(($($arg_name,)*));
                let num = Num::$name as u32;
                let result = unsafe { args.syscall(num) };
                SyscallResult::from_result(result)
            }
        )+
    }
}

#[cfg(feature = "kernel")]
macro_rules! syscalls {
    (
        $(
            $(#[$attrs:meta])*
            fn $name:ident($($arg_name:ident: $arg_ty:ty),*) -> $result:ty => $num:expr
        ),+
    ) => {
        use $crate::ErrNum;
        use $crate::marshal::{PackedArgs,SyscallArgs,SyscallResult};
        use core::fmt;

        pub trait HandleSyscall {
            #[allow(unused_variables)]
            fn log_entry(&self, name: &'static str, args: fmt::Arguments) { }

            #[allow(unused_variables)]
            fn log_exit(&self, name: &'static str, result: fmt::Arguments) { }

            $(
                fn $name<'a>(&self, $($arg_name: $arg_ty),*) -> Result<$result, ErrNum>;
            )+
        }

        pub fn dispatch<T: HandleSyscall>(handler: &T, num: usize, mut args: PackedArgs) -> isize {
            match num {
                $(
                    $num =>
                        (match SyscallArgs::from_args(&mut args) {
                            Ok(tuple) => {
                                handler.log_entry(stringify!($name), format_args!("{:?}", tuple));
                                let ($($arg_name,)*) = tuple;
                                let result = handler.$name($($arg_name),*);
                                handler.log_exit(stringify!($name), format_args!("{:?}", result));
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
