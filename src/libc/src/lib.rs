// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![no_std]
#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
    html_favicon_url = "https://doc.rust-lang.org/favicon.ico",
    html_root_url = "https://doc.rust-lang.org/nightly/",
    html_playground_url = "https://play.rust-lang.org/",
    issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/"
)]
#![cfg_attr(test, feature(test))]

//! Bindings for the C standard library and other platform libraries
//!
//! **NOTE:** These are *architecture and libc* specific. On Linux, these
//! bindings are only correct for glibc.
//!
//! This module contains bindings to the C standard library, organized into
//! modules by their defining standard.  Additionally, it contains some assorted
//! platform-specific definitions.  For convenience, most functions and types
//! are reexported, so `use libc::*` will import the available C bindings as
//! appropriate for the target platform. The exact set of functions available
//! are platform specific.
//!
//! *Note:* Because these definitions are platform-specific, some may not appear
//! in the generated documentation.
//!
//! We consider the following specs reasonably normative with respect to
//! interoperating with the C standard library (libc/msvcrt):
//!
//! * ISO 9899:1990 ('C95', 'ANSI C', 'Standard C'), NA1, 1995.
//! * ISO 9899:1999 ('C99' or 'C9x').
//! * ISO 9945:1988 / IEEE 1003.1-1988 ('POSIX.1').
//! * ISO 9945:2001 / IEEE 1003.1-2001 ('POSIX:2001', 'SUSv3').
//! * ISO 9945:2008 / IEEE 1003.1-2008 ('POSIX:2008', 'SUSv4').
//!
//! Note that any reference to the 1996 revision of POSIX, or any revs between
//! 1990 (when '88 was approved at ISO) and 2001 (when the next actual
//! revision-revision happened), are merely additions of other chapters (1b and
//! 1c) outside the core interfaces.
//!
//! Despite having several names each, these are *reasonably* coherent
//! point-in-time, list-of-definition sorts of specs. You can get each under a
//! variety of names but will wind up with the same definition in each case.
//!
//! See standards(7) in linux-manpages for more details.
//!
//! Our interface to these libraries is complicated by the non-universality of
//! conformance to any of them. About the only thing universally supported is
//! the first (C95), beyond that definitions quickly become absent on various
//! platforms.
//!
//! We therefore wind up dividing our module-space up (mostly for the sake of
//! sanity while editing, filling-in-details and eliminating duplication) into
//! definitions common-to-all (held in modules named c95, c99, posix88, posix01
//! and posix08) and definitions that appear only on *some* platforms (named
//! 'extra'). This would be things like significant OSX foundation kit, or Windows
//! library kernel32.dll, or various fancy glibc, Linux or BSD extensions.
//!
//! In addition to the per-platform 'extra' modules, we define a module of
//! 'common BSD' libc routines that never quite made it into POSIX but show up
//! in multiple derived systems. This is the 4.4BSD r2 / 1995 release, the final
//! one from Berkeley after the lawsuits died down and the CSRG dissolved.

#![allow(bad_style)]
#![cfg_attr(target_os = "nacl", allow(unused_imports))]
#[cfg(feature = "cargo-build")]
extern crate std as core;

#[cfg(test)]
extern crate std;
#[cfg(test)]
extern crate test;

#[link(name = "c")]
extern "C" {}

// Explicit export lists for the intersection (provided here) mean that
// you can write more-platform-agnostic code if you stick to just these
// symbols.

pub use crate::funcs::c95::stdlib::*;
pub use crate::types::common::c95::*;
pub use crate::types::common::c99::*;
pub use crate::types::common::posix88::*;
pub use crate::types::os::arch::c95::*;
pub use crate::types::os::arch::c99::*;
pub use crate::types::os::arch::extra::*;
pub use crate::types::os::arch::posix01::*;
pub use crate::types::os::arch::posix88::*;

// But we also reexport most everything
// if you're interested in writing platform-specific code.

// FIXME: This is a mess, but the design of this entire module needs to be
// reconsidered, so I'm not inclined to do better right now. As part of
// #11870 I removed all the pub globs here, leaving explicit reexports
// of everything that is actually used in-tree.
//
// So the following exports don't follow any particular plan.

pub mod types {

    // Types tend to vary *per architecture* so we pull their definitions out
    // into this module.

    // Standard types that are opaque or common, so are not per-target.
    pub mod common {
        pub mod c95 {
            /// Type used to construct void pointers for use with C.
            ///
            /// This type is only useful as a pointer target. Do not use it as a
            /// return type for FFI functions which have the `void` return type in
            /// C. Use the unit type `()` or omit the return type instead.
            ///
            /// For LLVM to recognize the void pointer type and by extension
            /// functions like malloc(), we need to have it represented as i8*
            /// in LLVM bitcode. The enum used here ensures this. We need two
            /// variants, because the compiler complains about the `repr`
            /// attribute otherwise.
            #[repr(u8)]
            pub enum c_void {
                #[doc(hidden)]
                __variant1,
                #[doc(hidden)]
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
        pub mod posix88 {
            pub enum DIR {}
            pub enum dirent_t {}
        }
        pub mod posix01 {}
        pub mod posix08 {}
        pub mod bsd44 {}
    }

    // Standard types that are scalar but vary by OS and arch.

    pub mod os {
        pub mod common {
            pub mod posix01 {
                use crate::types::common::c95::c_void;
                use crate::types::os::arch::c95::{c_char, c_long, c_ulong, size_t, suseconds_t, time_t};

                #[cfg(not(target_os = "nacl"))]
                pub type pthread_t = c_ulong;
                #[cfg(target_os = "nacl")]
                pub type pthread_t = *mut c_void;
                pub type rlim_t = u64;

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct glob_t {
                    pub gl_pathc: size_t,
                    pub gl_pathv: *mut *mut c_char,
                    pub gl_offs: size_t,

                    pub __unused1: *mut c_void,
                    pub __unused2: *mut c_void,
                    pub __unused3: *mut c_void,
                    pub __unused4: *mut c_void,
                    pub __unused5: *mut c_void,
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct timeval {
                    pub tv_sec: time_t,
                    pub tv_usec: suseconds_t,
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct timespec {
                    pub tv_sec: time_t,
                    pub tv_nsec: c_long,
                }

                pub enum timezone {}

                pub type sighandler_t = size_t;

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct rlimit {
                    pub rlim_cur: rlim_t,
                    pub rlim_max: rlim_t,
                }
            }

            pub mod bsd43 {
                use crate::types::os::arch::c95::c_long;
                use crate::types::os::common::posix01::timeval;
                // This is also specified in POSIX 2001, but only has two fields. All implementors
                // implement BSD 4.3 version.
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct rusage {
                    pub ru_utime: timeval,
                    pub ru_stime: timeval,
                    pub ru_maxrss: c_long,
                    pub ru_ixrss: c_long,
                    pub ru_idrss: c_long,
                    pub ru_isrss: c_long,
                    pub ru_minflt: c_long,
                    pub ru_majflt: c_long,
                    pub ru_nswap: c_long,
                    pub ru_inblock: c_long,
                    pub ru_oublock: c_long,
                    pub ru_msgsnd: c_long,
                    pub ru_msgrcv: c_long,
                    pub ru_nsignals: c_long,
                    pub ru_nvcsw: c_long,
                    pub ru_nivcsw: c_long,
                }
            }

            pub mod bsd44 {
                use crate::types::common::c95::c_void;
                use crate::types::os::arch::c95::{c_char, c_int, c_uint};

                pub type socklen_t = u32;
                pub type sa_family_t = u16;
                pub type in_port_t = u16;
                pub type in_addr_t = u32;
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct sockaddr {
                    pub sa_family: sa_family_t,
                    pub sa_data: [u8; 14],
                }
                #[repr(C)]
                #[derive(Copy)]
                pub struct sockaddr_storage {
                    pub ss_family: sa_family_t,
                    pub __ss_align: isize,
                    #[cfg(target_pointer_width = "32")]
                    pub __ss_pad2: [u8; 128 - 2 * 4],
                    #[cfg(target_pointer_width = "64")]
                    pub __ss_pad2: [u8; 128 - 2 * 8],
                }
                impl ::core::clone::Clone for sockaddr_storage {
                    fn clone(&self) -> sockaddr_storage {
                        *self
                    }
                }
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct sockaddr_in {
                    pub sin_family: sa_family_t,
                    pub sin_port: in_port_t,
                    pub sin_addr: in_addr,
                    pub sin_zero: [u8; 8],
                }
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct in_addr {
                    pub s_addr: in_addr_t,
                }
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct sockaddr_in6 {
                    pub sin6_family: sa_family_t,
                    pub sin6_port: in_port_t,
                    pub sin6_flowinfo: u32,
                    pub sin6_addr: in6_addr,
                    pub sin6_scope_id: u32,
                }
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct in6_addr {
                    pub s6_addr: [u16; 8],
                }
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct ip_mreq {
                    pub imr_multiaddr: in_addr,
                    pub imr_interface: in_addr,
                }
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct ip6_mreq {
                    pub ipv6mr_multiaddr: in6_addr,
                    pub ipv6mr_interface: c_uint,
                }
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct addrinfo {
                    pub ai_flags: c_int,
                    pub ai_family: c_int,
                    pub ai_socktype: c_int,
                    pub ai_protocol: c_int,
                    pub ai_addrlen: socklen_t,

                    #[cfg(target_os = "linux")]
                    pub ai_addr: *mut sockaddr,

                    #[cfg(target_os = "linux")]
                    pub ai_canonname: *mut c_char,

                    #[cfg(any(target_os = "android", target_os = "nacl"))]
                    pub ai_canonname: *mut c_char,

                    #[cfg(any(target_os = "android", target_os = "nacl"))]
                    pub ai_addr: *mut sockaddr,

                    pub ai_next: *mut addrinfo,
                }
                #[repr(C)]
                #[derive(Copy)]
                pub struct sockaddr_un {
                    pub sun_family: sa_family_t,
                    pub sun_path: [c_char; 108],
                }
                impl ::core::clone::Clone for sockaddr_un {
                    fn clone(&self) -> sockaddr_un {
                        *self
                    }
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct ifaddrs {
                    pub ifa_next: *mut ifaddrs,
                    pub ifa_name: *mut c_char,
                    pub ifa_flags: c_uint,
                    pub ifa_addr: *mut sockaddr,
                    pub ifa_netmask: *mut sockaddr,
                    pub ifa_ifu: *mut sockaddr, // FIXME This should be a union
                    pub ifa_data: *mut c_void,
                }
            }
        }

        #[cfg(any(
            target_arch = "x86",
            target_arch = "arm",
            target_arch = "mips",
            target_arch = "mipsel",
            target_arch = "powerpc",
            target_arch = "le32"
        ))]
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

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub enum jmp_buf {
                    __no1,
                    __no2,
                }
            }
            pub mod c99 {
                pub type c_longlong = i64;
                pub type c_ulonglong = u64;
                pub type intptr_t = i32;
                pub type uintptr_t = u32;
                pub type intmax_t = i64;
                pub type uintmax_t = u64;
            }
            #[cfg(any(
                target_arch = "mips",
                target_arch = "mipsel",
                target_arch = "powerpc",
                target_arch = "le32",
                all(any(target_arch = "arm", target_arch = "x86"), not(target_os = "android"))
            ))]
            pub mod posix88 {
                pub type off_t = i32;
                pub type dev_t = u64;
                pub type ino_t = u32;
                pub type pid_t = i32;
                pub type uid_t = u32;
                pub type gid_t = u32;
                pub type useconds_t = u32;
                pub type mode_t = u32;
                pub type ssize_t = i32;
            }
            #[cfg(all(any(target_arch = "arm", target_arch = "x86"), target_os = "android"))]
            pub mod posix88 {
                pub type off_t = i32;
                pub type dev_t = u32;
                pub type ino_t = u32;

                pub type pid_t = i32;
                pub type uid_t = u32;
                pub type gid_t = u32;
                pub type useconds_t = u32;

                pub type mode_t = u16;
                pub type ssize_t = i32;
            }
            #[cfg(any(
                all(any(target_arch = "arm", target_arch = "x86"), not(target_os = "android")),
                target_arch = "le32",
                target_arch = "powerpc"
            ))]
            pub mod posix01 {
                use crate::types::os::arch::c95::{c_long, c_short, time_t};
                use crate::types::os::arch::posix88::uid_t;
                use crate::types::os::arch::posix88::{dev_t, gid_t, ino_t};
                use crate::types::os::arch::posix88::{mode_t, off_t};

                pub type nlink_t = u32;
                pub type blksize_t = i32;
                pub type blkcnt_t = i32;

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct stat {
                    pub st_dev: dev_t,
                    pub __pad1: c_short,
                    pub st_ino: ino_t,
                    pub st_mode: mode_t,
                    pub st_nlink: nlink_t,
                    pub st_uid: uid_t,
                    pub st_gid: gid_t,
                    pub st_rdev: dev_t,
                    pub __pad2: c_short,
                    pub st_size: off_t,
                    pub st_blksize: blksize_t,
                    pub st_blocks: blkcnt_t,
                    pub st_atime: time_t,
                    pub st_atime_nsec: c_long,
                    pub st_mtime: time_t,
                    pub st_mtime_nsec: c_long,
                    pub st_ctime: time_t,
                    pub st_ctime_nsec: c_long,
                    pub __unused4: c_long,
                    pub __unused5: c_long,
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct utimbuf {
                    pub actime: time_t,
                    pub modtime: time_t,
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct pthread_attr_t {
                    pub __size: [u32; 9],
                }
            }

            #[cfg(all(any(target_arch = "arm", target_arch = "x86"), target_os = "android"))]
            pub mod posix01 {
                use types::os::arch::c95::{c_long, c_uchar, c_uint, c_ulong, time_t};
                use types::os::arch::c99::{c_longlong, c_ulonglong};
                use types::os::arch::posix88::{gid_t, uid_t};

                pub type nlink_t = u16;
                pub type blksize_t = u32;
                pub type blkcnt_t = u32;

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct stat {
                    pub st_dev: c_ulonglong,
                    pub __pad0: [c_uchar; 4],
                    pub __st_ino: c_long,
                    pub st_mode: c_uint,
                    pub st_nlink: c_uint,
                    pub st_uid: uid_t,
                    pub st_gid: gid_t,
                    pub st_rdev: c_ulonglong,
                    pub __pad3: [c_uchar; 4],
                    pub st_size: c_longlong,
                    pub st_blksize: c_ulong,
                    pub st_blocks: c_ulonglong,
                    pub st_atime: time_t,
                    pub st_atime_nsec: c_ulong,
                    pub st_mtime: time_t,
                    pub st_mtime_nsec: c_ulong,
                    pub st_ctime: time_t,
                    pub st_ctime_nsec: c_ulong,
                    pub st_ino: c_ulonglong,
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct utimbuf {
                    pub actime: time_t,
                    pub modtime: time_t,
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct pthread_attr_t {
                    pub __size: [u32; 9],
                }
            }

            #[cfg(any(target_arch = "mips", target_arch = "mipsel"))]
            pub mod posix01 {
                use types::os::arch::c95::{c_long, c_ulong, time_t};
                use types::os::arch::posix88::uid_t;
                use types::os::arch::posix88::{gid_t, ino_t};
                use types::os::arch::posix88::{mode_t, off_t};

                pub type nlink_t = u32;
                pub type blksize_t = i32;
                pub type blkcnt_t = i32;

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct stat {
                    pub st_dev: c_ulong,
                    pub st_pad1: [c_long; 3],
                    pub st_ino: ino_t,
                    pub st_mode: mode_t,
                    pub st_nlink: nlink_t,
                    pub st_uid: uid_t,
                    pub st_gid: gid_t,
                    pub st_rdev: c_ulong,
                    pub st_pad2: [c_long; 2],
                    pub st_size: off_t,
                    pub st_pad3: c_long,
                    pub st_atime: time_t,
                    pub st_atime_nsec: c_long,
                    pub st_mtime: time_t,
                    pub st_mtime_nsec: c_long,
                    pub st_ctime: time_t,
                    pub st_ctime_nsec: c_long,
                    pub st_blksize: blksize_t,
                    pub st_blocks: blkcnt_t,
                    pub st_pad5: [c_long; 14],
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct utimbuf {
                    pub actime: time_t,
                    pub modtime: time_t,
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct pthread_attr_t {
                    pub __size: [u32; 9],
                }
            }
            pub mod posix08 {}
            pub mod bsd44 {}
            pub mod extra {
                use crate::types::os::arch::c95::{c_int, c_uchar, c_ushort};
                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct sockaddr_ll {
                    pub sll_family: c_ushort,
                    pub sll_protocol: c_ushort,
                    pub sll_ifindex: c_int,
                    pub sll_hatype: c_ushort,
                    pub sll_pkttype: c_uchar,
                    pub sll_halen: c_uchar,
                    pub sll_addr: [c_uchar; 8],
                }
            }
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
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
                    pub rip: i64,
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
            pub mod posix88 {
                pub type off_t = i64;
                pub type dev_t = u64;
                pub type ino_t = u64;
                pub type pid_t = i32;
                pub type uid_t = u32;
                pub type gid_t = u32;
                pub type useconds_t = u32;
                pub type mode_t = u32;
                pub type ssize_t = i64;
            }
            #[cfg(not(target_arch = "aarch64"))]
            pub mod posix01 {
                use crate::types::os::arch::c95::{c_int, c_long, time_t};
                use crate::types::os::arch::posix88::uid_t;
                use crate::types::os::arch::posix88::{dev_t, gid_t, ino_t};
                use crate::types::os::arch::posix88::{mode_t, off_t};

                pub type nlink_t = u64;
                pub type blksize_t = i64;
                pub type blkcnt_t = i64;

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct stat {
                    pub st_dev: dev_t,
                    pub st_ino: ino_t,
                    pub st_nlink: nlink_t,
                    pub st_mode: mode_t,
                    pub st_uid: uid_t,
                    pub st_gid: gid_t,
                    pub __pad0: c_int,
                    pub st_rdev: dev_t,
                    pub st_size: off_t,
                    pub st_blksize: blksize_t,
                    pub st_blocks: blkcnt_t,
                    pub st_atime: time_t,
                    pub st_atime_nsec: c_long,
                    pub st_mtime: time_t,
                    pub st_mtime_nsec: c_long,
                    pub st_ctime: time_t,
                    pub st_ctime_nsec: c_long,
                    pub __unused: [c_long; 3],
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct utimbuf {
                    pub actime: time_t,
                    pub modtime: time_t,
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct pthread_attr_t {
                    pub __size: [u64; 7],
                }
            }
            #[cfg(target_arch = "aarch64")]
            pub mod posix01 {
                use types::os::arch::c95::{c_int, c_long, time_t};
                use types::os::arch::posix88::uid_t;
                use types::os::arch::posix88::{dev_t, gid_t, ino_t};
                use types::os::arch::posix88::{mode_t, off_t};

                pub type nlink_t = u32;
                pub type blksize_t = i32;
                pub type blkcnt_t = i64;

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct stat {
                    pub st_dev: dev_t,
                    pub st_ino: ino_t,
                    pub st_mode: mode_t,
                    pub st_nlink: nlink_t,
                    pub st_uid: uid_t,
                    pub st_gid: gid_t,
                    pub st_rdev: dev_t,
                    pub __pad1: dev_t,
                    pub st_size: off_t,
                    pub st_blksize: blksize_t,
                    pub __pad2: c_int,
                    pub st_blocks: blkcnt_t,
                    pub st_atime: time_t,
                    pub st_atime_nsec: c_long,
                    pub st_mtime: time_t,
                    pub st_mtime_nsec: c_long,
                    pub st_ctime: time_t,
                    pub st_ctime_nsec: c_long,
                    pub __unused: [c_int; 2],
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct utimbuf {
                    pub actime: time_t,
                    pub modtime: time_t,
                }

                #[repr(C)]
                #[derive(Copy, Clone)]
                pub struct pthread_attr_t {
                    pub __size: [u64; 8],
                }
            }
            pub mod posix08 {}
            pub mod bsd44 {}
            pub mod extra {
                use crate::types::os::arch::c95::{c_int, c_uchar, c_ushort};
                #[derive(Copy, Clone)]
                pub struct sockaddr_ll {
                    pub sll_family: c_ushort,
                    pub sll_protocol: c_ushort,
                    pub sll_ifindex: c_int,
                    pub sll_hatype: c_ushort,
                    pub sll_pkttype: c_uchar,
                    pub sll_halen: c_uchar,
                    pub sll_addr: [c_uchar; 8],
                }
            }
        }
    }
}

pub mod funcs {
    pub mod c95 {
        pub mod stdlib {
            use crate::types::common::c95::c_void;
            use crate::types::os::arch::c95::{c_char, c_int, jmp_buf, size_t};

            extern "C" {
                pub fn malloc(size: size_t) -> *mut c_void;
                pub fn realloc(p: *mut c_void, size: size_t) -> *mut c_void;
                pub fn free(p: *mut c_void);
                pub fn setjmp(env: *mut jmp_buf) -> c_int;
                pub fn longjmp(env: *const jmp_buf, val: c_int) -> !;
                pub fn memchr(cx: *const c_void, c: c_int, n: size_t) -> *mut c_void;
                pub fn strlen(p: *const c_char) -> size_t;
            }
        }
    }
}
