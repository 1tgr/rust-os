use crate::detail::UntypedRecursiveMutex;
use crate::Thread;
use core::fmt;
use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering};
use libc::{c_char, c_int, c_void, mode_t, off_t, size_t, ssize_t};
use syscall::{Handle, Result};

#[allow(non_upper_case_globals)]
#[thread_local]
static mut errno: c_int = 0;

#[allow(non_upper_case_globals)]
pub static stdin: Handle = 0;

#[allow(non_upper_case_globals)]
pub static stdout: Handle = 1;

pub struct StdoutWriter;

impl fmt::Write for StdoutWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match syscall::write(stdout, s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}

pub struct StderrWriter;

impl fmt::Write for StderrWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match syscall::write(stdout, s.as_bytes()) {
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
    const HEAP_SIZE: usize = 16 * 1048 * 1048;
    static mut HEAP: [u8; HEAP_SIZE] = [0; 16 * 1048 * 1048];
    static BRK: AtomicUsize = AtomicUsize::new(0);
    let prev_brk = BRK.fetch_add(len, Ordering::SeqCst);
    assert!(prev_brk + len <= HEAP_SIZE, "{} + {} > {}", prev_brk, len, HEAP_SIZE);
    unsafe { &mut HEAP[prev_brk] }
}

static mut MALLOC_LOCK: Option<UntypedRecursiveMutex> = None;

#[no_mangle]
pub extern "C" fn __malloc_lock(_reent: *mut c_void) {
    let mutex = unsafe { MALLOC_LOCK.as_ref().unwrap() };
    mutex.lock();
}

#[no_mangle]
pub extern "C" fn __malloc_unlock(_reent: *mut c_void) {
    let mutex = unsafe { MALLOC_LOCK.as_ref().unwrap() };
    mutex.unlock();
}

#[no_mangle]
pub unsafe extern "C" fn __assert_fail(
    _assertion: *const c_char,
    _file: *const c_char,
    _line: c_int,
    _function: *const c_char,
) -> ! {
    Thread::exit(-1)
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
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    if fd != 2 {
        panic!("write({}, {:?}, {})", fd, buf, count);
    }

    let buf = slice::from_raw_parts(buf as *const u8, count as usize);
    syscall::write(stdout, buf).map(|n| n as ssize_t).unwrap_or(-1)
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
    MALLOC_LOCK = Some(UntypedRecursiveMutex::new());

    extern "C" {
        static __ctors_start: extern "C" fn();
        static __ctors_end: extern "C" fn();
    }

    let mut ptr = &__ctors_start as *const _;
    while ptr < &__ctors_end {
        (*ptr)();
        ptr = ptr.offset(1);
    }

    Ok(())
}

pub unsafe fn shutdown(code: i32) -> ! {
    MALLOC_LOCK.as_ref().take();
    Thread::exit(code)
}
