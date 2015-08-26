use marshal::FileHandle;

syscalls! {
    0 => exit_thread(code: i32) -> (),
    1 => alloc_pages(len: usize) -> *mut u8,
    2 => free_pages(ptr: *mut u8) -> bool,
    3 => write(file: FileHandle, bytes: &'a [u8]) -> (),
    4 => read(file: FileHandle, buf: &'a mut [u8]) -> usize
}
