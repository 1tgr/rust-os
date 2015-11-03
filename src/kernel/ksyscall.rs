use alloc::arc::Arc;
use arch::vga_bochs;
use console::Console;
use io::{Read,Write};
use logging::Writer;
use prelude::*;
use process::{self,KObj,SharedMemBlock};
use singleton::{DropSingleton,Singleton};
use syscall::{self,ErrNum,Handle,HandleSyscall,PackedArgs,Result};
use thread;

pub struct SyscallHandler {
    console: Arc<Console>
}

static HANDLER: Singleton<SyscallHandler> = Singleton::new();

pub type DropSyscallHandler = DropSingleton<SyscallHandler>;

pub fn register_handler(handler: SyscallHandler) -> DropSyscallHandler {
    HANDLER.register(handler)
}

pub fn dispatch(num: usize, args: PackedArgs) -> isize {
    if let Some(handler) = HANDLER.get() {
        syscall::dispatch(handler, &mut Writer, num, args)
    } else {
        0
    }
}

impl SyscallHandler {
    pub fn new(console: Arc<Console>) -> Self {
        SyscallHandler {
            console: console
        }
    }
}

impl HandleSyscall for SyscallHandler {
    fn exit_thread(&self, code: i32) -> Result<()> {
        thread::exit(code)
    }

    fn write(&self, file: Handle, bytes: &[u8]) -> Result<usize> {
        let file = try!(process::resolve_handle(file, |kobj| kobj.write()));
        file.write(bytes)
    }

    fn read(&self, file: Handle, buf: &mut [u8]) -> Result<usize> {
        let file = try!(process::resolve_handle(file, |kobj| kobj.read()));
        file.read(buf)
    }

    fn alloc_pages(&self, len: usize) -> Result<*mut u8> {
        match process::alloc(len, true, true) {
            Ok(slice) => Ok(slice.as_mut_ptr()),
            Err(_) => Err(ErrNum::OutOfMemory)
        }
    }

    fn free_pages(&self, p: *mut u8) -> Result<bool> {
        Ok(process::free(p))
    }

    fn open(&self, filename: &str) -> Result<Handle> {
        let file: Arc<KObj> =
            match filename {
                "stdin" => self.console.clone(),
                "stdout" => self.console.clone(),
                _ => { return Err(ErrNum::FileNotFound) }
            };

        Ok(process::make_handle(file))
    }

    fn close(&self, handle: Handle) -> Result<()> {
        if !process::close_handle(handle) {
            return Err(ErrNum::InvalidHandle)
        }

        Ok(())
    }

    fn init_video_mode(&self, width: u16, height: u16, bpp: u8) -> Result<*mut u8> {
        let slice = try!(vga_bochs::init(width, height, bpp));
        Ok(slice.as_mut_ptr())
    }

    fn spawn(&self, executable: &str) -> Result<Handle> {
        let executable = String::from(executable);
        let (_, deferred) = try!(process::spawn(executable));
        Ok(process::make_handle(Arc::new(deferred)))
    }

    fn wait_for_exit(&self, process: Handle) -> Result<i32> {
        let deferred = try!(process::resolve_handle(process, |kobj| kobj.deferred_i32()));
        Ok((*deferred).clone().get())
    }

    fn create_shared_mem(&self) -> Result<Handle> {
        Ok(process::make_handle(Arc::new(SharedMemBlock::new())))
    }

    fn map_shared_mem(&self, block: Handle, len: usize, writable: bool) -> Result<*mut u8> {
        let block = try!(process::resolve_handle(block, |kobj| kobj.shared_mem_block()));
        let slice = try!(process::map_shared(block, len, true, writable));
        Ok(slice.as_mut_ptr())
    }
}
