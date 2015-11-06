use super::Handle;

syscalls! {
    /// Exits the current thread.
    fn exit_thread(code: i32) -> () => 0,

    /// Allocates memory.
    fn alloc_pages(len: usize) -> *mut u8 => 1,

    /// Frees memory allocated by `alloc_pages`.
    fn free_pages(ptr: *mut u8) -> bool => 2,

    /// Opens a file.
    fn open(filename: &'a str) -> Handle => 3,

    /// Closes a handle.
    fn close(file: Handle) -> () => 4,

    /// Writes to a file.
    fn write(file: Handle, buf: &'a [u8]) -> usize => 5,

    /// Reads from a file.
    fn read(file: Handle, buf: &'a mut [u8]) -> usize => 6,

    fn init_video_mode(width: u16, height: u16, bpp: u8) -> *mut u8 => 7,
    fn spawn(executable: &str, inherit: &'a [Handle]) -> Handle => 8,
    fn wait_for_exit(process: Handle) -> i32 => 9,
    fn create_shared_mem() -> Handle => 10,
    fn map_shared_mem(block: Handle, len: usize, writable: bool) -> *mut u8 => 11
}
