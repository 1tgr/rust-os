mod file;
mod oshandle;
mod osmem;
mod sharedmem;

pub use self::file::*;
pub use self::oshandle::*;
pub use self::osmem::*;
pub use self::sharedmem::*;

use syscall;

pub type Result<T> = syscall::Result<T>;
