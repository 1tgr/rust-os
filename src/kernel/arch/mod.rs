#[cfg(target_arch = "arm")]
mod arm32;

#[cfg(target_arch = "arm")]
pub use arm32::*;

#[cfg(target_arch = "x86_64")]
mod x86_common;

#[cfg(target_arch = "x86_64")]
mod amd64;

#[cfg(target_arch = "x86_64")]
pub use amd64::*;
