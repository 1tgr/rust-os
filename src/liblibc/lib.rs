#![crate_name = "libc"]

#![feature(no_std)]
#![no_std]

#![allow(non_camel_case_types)]

pub use funcs::c95::stdlib::*;
pub use types::common::c95::*;
pub use types::common::c99::*;
pub use types::os::arch::c95::*;
pub use types::os::arch::c99::*;

pub mod types {
    pub mod common {
        pub mod c95 {
            /// Type used to construct void pointers for use with C.
            ///
            /// This type is only useful as a pointer target. Do not use it as a
            /// return type for FFI functions which have the `void` return type in
            /// C. Use the unit type `()` or omit the return type instead.
            ///
            /// For LLVM to recognize the void pointer type and by extension
            /// functions like malloc(), we need to have it represented as i8* in
            /// LLVM bitcode. The enum used here ensures this and prevents misuse
            /// of the "raw" type by only having private variants.. We need two
            /// variants, because the compiler complains about the repr attribute
            /// otherwise.
            #[repr(u8)]
            pub enum c_void {
                __variant1,
                __variant2,
            }

            pub enum FILE {}
            pub enum fpos_t {}
        }
        pub mod c99 {
            pub type int8_t = i8;
            pub type int16_t = i16;
            pub type int32_t = i32;
            pub type int64_t = i64;
            pub type uint8_t = u8;
            pub type uint16_t = u16;
            pub type uint32_t = u32;
            pub type uint64_t = u64;
        }
    }

    pub mod os {
        #[cfg(any(target_arch = "x86",
                  target_arch = "arm",
                  target_arch = "mips",
                  target_arch = "mipsel",
                  target_arch = "powerpc",
                  target_arch = "le32"))]
        pub mod arch {
            pub mod c95 {
                pub type c_char = i8;
                pub type c_schar = i8;
                pub type c_uchar = u8;
                pub type c_short = i16;
                pub type c_ushort = u16;
                pub type c_int = i32;
                pub type c_uint = u32;
                pub type c_long = i32;
                pub type c_ulong = u32;
                pub type c_float = f32;
                pub type c_double = f64;
                pub type size_t = u32;
                pub type ptrdiff_t = i32;
                pub type clock_t = i32;
                pub type time_t = i32;
                pub type suseconds_t = i32;
                pub type wchar_t = i32;
            }
            pub mod c99 {
                pub type c_longlong = i64;
                pub type c_ulonglong = u64;
                pub type intptr_t = i32;
                pub type uintptr_t = u32;
                pub type intmax_t = i64;
                pub type uintmax_t = u64;
            }
        }

        #[cfg(any(target_arch = "x86_64",
                  target_arch = "aarch64"))]
        pub mod arch {
            pub mod c95 {
                #[cfg(not(target_arch = "aarch64"))]
                pub type c_char = i8;
                #[cfg(target_arch = "aarch64")]
                pub type c_char = u8;
                pub type c_schar = i8;
                pub type c_uchar = u8;
                pub type c_short = i16;
                pub type c_ushort = u16;
                pub type c_int = i32;
                pub type c_uint = u32;
                pub type c_long = i64;
                pub type c_ulong = u64;
                pub type c_float = f32;
                pub type c_double = f64;
                pub type size_t = u64;
                pub type ptrdiff_t = i64;
                pub type clock_t = i64;
                pub type time_t = i64;
                pub type suseconds_t = i64;
                #[cfg(not(target_arch = "aarch64"))]
                pub type wchar_t = i32;
                #[cfg(target_arch = "aarch64")]
                pub type wchar_t = u32;

                /*
                 **  jmp_buf:
                 **   rbx rbp r12 r13 r14 r15 rsp rip
                 **   0   8   16  24  32  40  48  56
                 */

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct jmp_buf {
                    pub rbx: i64,
                    pub rbp: i64,
                    pub r12: i64,
                    pub r13: i64,
                    pub r14: i64,
                    pub r15: i64,
                    pub rsp: i64,
                    pub rip: i64
                }

            }
            pub mod c99 {
                pub type c_longlong = i64;
                pub type c_ulonglong = u64;
                pub type intptr_t = i64;
                pub type uintptr_t = u64;
                pub type intmax_t = i64;
                pub type uintmax_t = u64;
            }
        }
    }
}

pub mod funcs {
    pub mod c95 {
        pub mod stdlib {
            use types::common::c95::{c_void};
            use types::os::arch::c95::{jmp_buf,size_t,c_int};

            extern {
                pub fn malloc(size: size_t) -> *mut c_void;
                pub fn realloc(p: *mut c_void, size: size_t) -> *mut c_void;
                pub fn free(p: *mut c_void);
                pub fn setjmp(env: *mut jmp_buf) -> c_int;
                pub fn longjmp(env: *const jmp_buf, val: c_int) -> !;
            }
        }
    }
}
