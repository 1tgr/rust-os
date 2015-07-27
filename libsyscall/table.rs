syscalls! {
    0 => write(&'a str) -> (),
    1 => exit_thread(u32) -> (),
    2 => read_line(&'a mut [u8]) -> usize
}
