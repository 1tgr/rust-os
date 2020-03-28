use syscall::libc_helpers;

#[lang = "start"]
fn dummy_start(_main: *const u8, _argc: isize, _argv: *const *const u8) -> isize {
    panic!("dummy_start was called")
}

extern "C" {
    fn start(argc: isize, argv: *const *const u8) -> isize;
}

#[no_mangle]
pub unsafe extern "C" fn entry() {
    let result = libc_helpers::init().and_then(|()| Ok(start(0, 0 as *const _)));

    let code = match result {
        Ok(code) => code,
        Err(num) => -(num as isize),
    };

    libc_helpers::shutdown(code as i32);
}
