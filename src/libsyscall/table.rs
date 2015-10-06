use marshal::FileHandle;

syscalls! {
    0 => exit_thread(code: i32) -> (),
    1 => alloc_pages(len: usize) -> *mut u8,
    2 => free_pages(ptr: *mut u8) -> bool,
    3 => open(filename: &'a str) -> FileHandle,
    4 => close(file: FileHandle) -> (),
    5 => write(file: FileHandle, buf: &'a [u8]) -> usize,
    6 => read(file: FileHandle, buf: &'a mut [u8]) -> usize
}
