#[cfg(not(feature = "kernel"))]
macro_rules! syscalls {
    (
        $(
            $(#[$attrs:meta])*
            fn $name:ident($($arg_name:ident: $arg_ty:ty),+) -> $result:ty => $num:expr
        ),+
    ) => {
        use $crate::ErrNum;

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
        use $crate::ErrNum;
        use $crate::marshal::{PackedArgs,SyscallArgs,SyscallResult};
        use core::fmt::Write;

        pub trait HandleSyscall {
            $(
                fn $name<'a>(&self, $($arg_name: $arg_ty),+) -> Result<$result, ErrNum>;
            )+
        }

        pub fn dispatch<T: HandleSyscall, W: Write>(handler: &T, writer: &mut W, num: usize, mut args: PackedArgs) -> isize {
            match num {
                $(
                    $num =>
                        (match SyscallArgs::from_args(&mut args) {
                            Ok(tuple) => {
                                let _ = write!(writer, concat!(stringify!($name), "{:?}"), tuple);
                                let ($($arg_name,)+) = tuple;
                                let result = handler.$name($($arg_name),+);
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
