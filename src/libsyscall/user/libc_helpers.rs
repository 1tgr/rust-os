use libc::{c_char,c_int,c_void,size_t,ssize_t,off_t,mode_t};
use table as syscall;
use ::{Handle,Result};

static mut ERRNO: u32 = 0;

extern {
    static init_array_start: extern "C" fn();
    static init_array_end: extern "C" fn();
}

static mut errno: c_int = 0;
pub static mut stdin: Handle = 0;
pub static mut stdout: Handle = 0;

#[no_mangle]
pub unsafe extern fn __errno() -> *mut c_int {
    &mut errno
}

#[no_mangle]
pub extern fn sbrk(len: usize) -> *mut u8 {
    match syscall::alloc_pages(len) {
        Ok(p) => p,
        Err(num) => {
            unsafe { ERRNO = num as u32; }
            0 as *mut u8
        }
    }
}

#[no_mangle]
pub unsafe extern fn __assert_fail(_assertion: *const c_char, _file: *const c_char, _line: c_int, _function: *const c_char) -> ! {
    let _ = syscall::exit_thread(-1);
    unreachable!()
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

#[no_mangle]
pub extern fn open(_path: *const c_char, _oflag: c_int, _mode: mode_t) -> c_int {
    panic!("open was called")
}

#[no_mangle]
pub unsafe extern fn unlink(_c: *const c_char) -> c_int {
    panic!("unlink was called")
}

unsafe fn init() -> Result<()> {
    let mut ptr = &init_array_start as *const _;
    while ptr < &init_array_end {
        (*ptr)();
        ptr = ptr.offset(1);
    }

    stdin = try!(syscall::open("stdin"));
    stdout = try!(syscall::open("stdout"));
    Ok(())
}

extern {
    fn main() -> i32;
}

#[no_mangle]
pub unsafe extern fn start() {
    let result = init().and_then(|()| Ok(main()));
    let _ = syscall::close(stdin);
    let _ = syscall::close(stdout);

    let code =
        match result {
            Ok(code) => code,
            Err(num) => -(num as i32)
        };

    let _ = syscall::exit_thread(code);
}
