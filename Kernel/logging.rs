use ::arch::debug;
use core::fmt::{Error,Write};
use spin::{MutexGuard,StaticMutex,STATIC_MUTEX_INIT};
use std::result::Result::{self,Ok};

pub struct Writer<'a> {
    state: MutexGuard<'a, ()>
}

static LOGGING_LOCK: StaticMutex = STATIC_MUTEX_INIT;

impl<'a> Writer<'a> {
	pub fn get(module: &str) -> Writer {
		let mut ret = Writer { state: LOGGING_LOCK.lock() };
		
		{
			use core::fmt::Write;
			let _ = write!(&mut ret, "[{}] ", module);
		}
		
		ret
	}
}

impl<'a> Write for Writer<'a>
{
	fn write_str(&mut self, s: &str) -> Result<(), Error> {
        debug::puts(s);
		Ok(())
	}
}

