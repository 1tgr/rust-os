use arch::debug;
use core::fmt::{Error,Write};

#[derive(Clone)]
pub struct Writer;

impl Writer {
	pub fn get(module: &str) -> Writer {
        use core::fmt::Write;
        let mut writer = Writer;
        let _ = write!(&mut writer, "[{}] ", module);
        writer
	}
}

impl Write for Writer {
	fn write_str(&mut self, s: &str) -> Result<(), Error> {
        debug::puts(s);
		Ok(())
	}
}
