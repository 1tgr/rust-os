use os::{libc_helpers, Termination};

#[lang = "start"]
unsafe fn lang_start<T>(main: fn() -> T, _argc: isize, _argv: *const *const u8) -> isize
where
    T: Termination + 'static,
{
    let code = libc_helpers::init().map(|()| main().report()).report();
    libc_helpers::shutdown(code as i32);
}
