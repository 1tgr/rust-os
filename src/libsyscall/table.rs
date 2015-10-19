use marshal::FileHandle;

syscalls! {
    /// Exits the current thread.
    fn exit_thread(code: i32) -> () => 0,

    /// Allocates memory.
    fn alloc_pages(len: usize) -> *mut u8 => 1,

    /// Frees memory allocated by `alloc_pages`.
    fn free_pages(ptr: *mut u8) -> bool => 2,

    /// Opens a file.
    fn open(filename: &'a str) -> FileHandle => 3,

    /// Closes a file opened by `open`.
    fn close(file: FileHandle) -> () => 4,

    /// Writes to a file.
    fn write(file: FileHandle, buf: &'a [u8]) -> usize => 5,

    /// Reads from a file.
    fn read(file: FileHandle, buf: &'a mut [u8]) -> usize => 6,

    fn init_video_mode(width: u16, height: u16, bpp: u8) -> *mut u8 => 7
}
