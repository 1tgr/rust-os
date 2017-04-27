use arch::debug;
use core::fmt::Write;
use core::mem;
use libc::{c_char,c_int,c_void,size_t,ssize_t,off_t};
use logging::Writer;

extern {
    static init_array_start: extern "C" fn();
    static init_array_end: extern "C" fn();
}

#[allow(non_upper_case_globals)]
static mut errno: c_int = 0;

#[no_mangle]
pub unsafe extern fn __errno() -> *mut c_int {
    &mut errno
}

#[no_mangle]
pub unsafe extern fn __assert_fail(assertion: *const c_char, file: *const c_char, line: c_int, function: *const c_char) -> ! {
    let mut writer = Writer::get(module_path!());
    debug::put_cstr(file);
    let _ = write!(&mut writer, "({}): in function ", line);
    debug::put_cstr(function);
    let _ = write!(&mut writer, ": ");
    debug::put_cstr(assertion);
    mem::drop(writer);
    panic!("assertion failed in C code");
}

#[no_mangle]
pub extern fn __stack_chk_fail() -> ! {
    panic!("__stack_chk_fail was called")
}

#[no_mangle]
pub extern fn _fputwc_r(_ptr: *mut c_void, _wc: char, _fp: *mut c_void) -> c_int {
    panic!("_fputwc_r was called")
}

#[no_mangle]
pub extern fn _exit(_n: c_int) -> ! {
    panic!("_exit was called")
}

#[no_mangle]
pub extern fn kill(_pid: c_int, _sig: c_int) -> c_int {
    panic!("kill was called")
}

#[no_mangle]
pub extern fn getpid() -> c_int {
    panic!("getpid was called")
}

#[no_mangle]
pub extern fn write(_fd: c_int, _buf: *const c_void, _count: size_t) -> ssize_t {
    panic!("write was called")
}

#[no_mangle]
pub extern fn close(_fd: c_int) -> c_int {
    panic!("close was called")
}

#[no_mangle]
pub extern fn fstat(_fd: c_int, _buf: *mut c_void) -> c_int {
    panic!("fstat was called")
}

#[no_mangle]
pub extern fn isatty(_fd: c_int) -> c_int {
    panic!("isatty was called")
}

#[no_mangle]
pub extern fn lseek(_fd: c_int, _offset: off_t, _whence: c_int) -> off_t {
    panic!("lseek was called")
}

#[no_mangle]
pub extern fn read(_fd: c_int, _buf: *mut c_void, _count: size_t) -> ssize_t {
    panic!("read was called")
}

pub unsafe fn init() {
    let mut ptr = &init_array_start as *const _;
    while ptr < &init_array_end {
        (*ptr)();
        ptr = ptr.offset(1);
    }
}
