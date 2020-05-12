use crate::table as syscall;
use crate::{Handle, Result};
use core::fmt;
use core::sync::atomic::{AtomicUsize, Ordering};
use libc::{c_char, c_int, c_void, mode_t, off_t, size_t, ssize_t};

static mut ERRNO: u32 = 0;

extern "C" {
    static init_array_start: extern "C" fn();
    static init_array_end: extern "C" fn();
}

#[allow(non_upper_case_globals)]
static mut errno: c_int = 0;

#[allow(non_upper_case_globals)]
pub static mut stdin: Handle = 0;

#[allow(non_upper_case_globals)]
pub static mut stdout: Handle = 0;

pub struct StdoutWriter;

impl fmt::Write for StdoutWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match syscall::write(unsafe { stdout }, s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}

pub struct StderrWriter;

impl fmt::Write for StderrWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match syscall::write(unsafe { stdout }, s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn __errno() -> *mut c_int {
    &mut errno
}

#[no_mangle]
pub extern "C" fn sbrk(len: usize) -> *mut u8 {
    match syscall::alloc_pages(len) {
        Ok(p) => p,
        Err(num) => {
            unsafe {
                ERRNO = num as u32;
            }
            0 as *mut u8
        }
    }
}

static MALLOC_OWNER: AtomicUsize = AtomicUsize::new(0);
static mut MALLOC_LOCK_COUNT: usize = 0;
static mut MALLOC_LOCK: Handle = 0;

#[no_mangle]
pub extern "C" fn __malloc_lock(_reent: *mut c_void) {
    let current_thread_id = syscall::current_thread_id().unwrap();
    assert_ne!(current_thread_id, 0);

    if MALLOC_OWNER.load(Ordering::SeqCst) != current_thread_id {
        syscall::lock_mutex(unsafe { MALLOC_LOCK }).unwrap();
        while MALLOC_OWNER.compare_and_swap(0, current_thread_id, Ordering::SeqCst) != 0 {
            syscall::unlock_mutex(unsafe { MALLOC_LOCK }).unwrap();
            syscall::lock_mutex(unsafe { MALLOC_LOCK }).unwrap();
        }
    }

    unsafe {
        MALLOC_LOCK_COUNT += 1;
    }
}

#[no_mangle]
pub extern "C" fn __malloc_unlock(_reent: *mut c_void) {
    let prev_count;
    unsafe {
        prev_count = MALLOC_LOCK_COUNT;
        MALLOC_LOCK_COUNT -= 1;
    }

    if prev_count == 1 {
        let current_thread_id = syscall::current_thread_id().unwrap();
        assert_eq!(MALLOC_OWNER.swap(0, Ordering::SeqCst), current_thread_id);
        syscall::unlock_mutex(unsafe { MALLOC_LOCK }).unwrap();
    }
}

#[no_mangle]
pub unsafe extern "C" fn __assert_fail(
    _assertion: *const c_char,
    _file: *const c_char,
    _line: c_int,
    _function: *const c_char,
) -> ! {
    let _ = syscall::exit_thread(-1);
    unreachable!()
}

#[no_mangle]
pub extern "C" fn __stack_chk_fail() -> ! {
    panic!("__stack_chk_fail was called")
}

#[no_mangle]
pub extern "C" fn _fputwc_r(_ptr: *mut c_void, _wc: char, _fp: *mut c_void) -> c_int {
    panic!("_fputwc_r was called")
}

#[no_mangle]
pub extern "C" fn _exit(_n: c_int) -> ! {
    panic!("_exit was called")
}

#[no_mangle]
pub extern "C" fn kill(_pid: c_int, _sig: c_int) -> c_int {
    panic!("kill was called")
}

#[no_mangle]
pub extern "C" fn getpid() -> c_int {
    panic!("getpid was called")
}

#[no_mangle]
pub extern "C" fn write(_fd: c_int, _buf: *const c_void, _count: size_t) -> ssize_t {
    panic!("write was called")
}

#[no_mangle]
pub extern "C" fn close(_fd: c_int) -> c_int {
    panic!("close was called")
}

#[no_mangle]
pub extern "C" fn fstat(_fd: c_int, _buf: *mut c_void) -> c_int {
    panic!("fstat was called")
}

#[no_mangle]
pub extern "C" fn isatty(_fd: c_int) -> c_int {
    panic!("isatty was called")
}

#[no_mangle]
pub extern "C" fn lseek(_fd: c_int, _offset: off_t, _whence: c_int) -> off_t {
    panic!("lseek was called")
}

#[no_mangle]
pub extern "C" fn read(_fd: c_int, _buf: *mut c_void, _count: size_t) -> ssize_t {
    panic!("read was called")
}

#[no_mangle]
pub extern "C" fn open(_path: *const c_char, _oflag: c_int, _mode: mode_t) -> c_int {
    panic!("open was called")
}

#[no_mangle]
pub unsafe extern "C" fn unlink(_c: *const c_char) -> c_int {
    panic!("unlink was called")
}

pub unsafe fn init() -> Result<()> {
    MALLOC_LOCK = syscall::create_mutex()?;

    let mut ptr = &init_array_start as *const _;
    while ptr < &init_array_end {
        (*ptr)();
        ptr = ptr.offset(1);
    }

    stdin = syscall::open("stdin")?;
    stdout = syscall::open("stdout")?;
    Ok(())
}

pub unsafe fn shutdown(code: i32) -> ! {
    let _ = syscall::close(stdin);
    let _ = syscall::close(stdout);
    let _ = syscall::close(MALLOC_LOCK);
    let _ = syscall::exit_thread(code);
    unreachable!()
}
