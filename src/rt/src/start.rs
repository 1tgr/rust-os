use os::libc_helpers;
use os::Result;

#[lang = "termination"]
trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        Ok(()).report()
    }
}

impl Termination for Result<()> {
    fn report(self) -> i32 {
        self.map(|()| 0).report()
    }
}

impl Termination for Result<i32> {
    fn report(self) -> i32 {
        self.unwrap_or_else(|num| -(num as i32))
    }
}

#[lang = "start"]
unsafe fn lang_start<T: Termination + 'static>(main: fn() -> T, _argc: isize, _argv: *const *const u8) -> isize {
    let code = libc_helpers::init().map(|()| main().report()).report();
    libc_helpers::shutdown(code as i32);
}
