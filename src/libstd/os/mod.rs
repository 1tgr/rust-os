#![stable(feature = "rust-os", since = "1.0.0")]

mod file;
mod oshandle;
mod osmem;
mod sharedmem;

#[stable(feature = "rust-os", since = "1.0.0")]
pub use self::file::*;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use self::oshandle::*;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use self::osmem::*;
#[stable(feature = "rust-os", since = "1.0.0")]
pub use self::sharedmem::*;

use syscall;

#[stable(feature = "rust-os", since = "1.0.0")]
pub type Result<T> = syscall::Result<T>;
