syscalls! {
    0 => write(&'a str) -> (),
    1 => exit_thread(i32) -> (),
    2 => read_line(&'a mut [u8]) -> usize,
    3 => alloc_pages(usize) -> *mut u8,
    4 => free_pages(*mut u8) -> bool
}
