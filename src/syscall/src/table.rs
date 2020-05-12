use crate::{Handle, Result};

syscalls! {
    /// Exits the current thread.
    fn exit_thread(code: i32) -> ! => 0,

    /// Allocates memory.
    fn alloc_pages(len: usize) -> Result<*mut u8> => 1,

    /// Frees memory allocated by `alloc_pages`.
    fn free_pages(ptr: *mut u8) -> bool => 2,

    /// Opens a file.
    fn open(filename: &'a str) -> Result<Handle> => 3,

    /// Closes a handle.
    fn close(file: Handle) -> Result<()> => 4,

    /// Writes to a file.
    fn write(file: Handle, buf: &'a [u8]) -> Result<usize> => 5,

    /// Reads from a file.
    fn read(file: Handle, buf: &'a mut [u8]) -> Result<usize> => 6,

    fn init_video_mode(width: u16, height: u16, bpp: u8) -> Result<*mut u8> => 7,
    fn spawn_process(executable: &str, inherit: &'a [Handle]) -> Result<Handle> => 8,
    fn wait_for_exit(process: Handle) -> Result<i32> => 9,
    fn create_shared_mem() -> Handle => 10,
    fn map_shared_mem(block: Handle, len: usize, writable: bool) -> Result<*mut u8> => 11,
    fn create_pipe() -> Handle => 12,
    fn open_handle(from_process: Handle, from_handle: usize) -> Result<Handle> => 13,
    fn create_mutex() -> Handle => 14,
    fn lock_mutex(mutex: Handle) -> Result<()> => 15,
    fn unlock_mutex(mutex: Handle) -> Result<()> => 16,
    fn spawn_thread(entry: extern fn(usize), context: usize) -> Handle => 17,
    fn schedule() -> () => 18,
    fn current_thread_id() -> usize => 19
}
