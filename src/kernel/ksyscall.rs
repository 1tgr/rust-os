use alloc::arc::Arc;
use arch::vga_bochs;
use console::Console;
use core::fmt::{self,Write};
use io::Pipe;
use kobj::KObj;
use logging::Writer;
use prelude::*;
use process::{self,SharedMemBlock};
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
        syscall::dispatch(handler, num, args)
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
    fn log_entry(&self, msg: fmt::Arguments) {
        let _ = write!(&mut Writer, "[{}] {}", thread::current_process().name(), msg);
    }

    fn log_exit(&self, msg: fmt::Arguments) {
        let _ = writeln!(&mut Writer, " => {:?}", msg);
    }

    fn exit_thread(&self, code: i32) -> Result<()> {
        thread::exit(code)
    }

    fn write(&self, file: Handle, bytes: &[u8]) -> Result<usize> {
        let file = process::resolve_handle_ref(file, |kobj| kobj.write())?;
        file.write(bytes)
    }

    fn read(&self, file: Handle, buf: &mut [u8]) -> Result<usize> {
        let file = process::resolve_handle_ref(file, |kobj| kobj.read())?;
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
        let slice = vga_bochs::init(width, height, bpp)?;
        Ok(slice.as_mut_ptr())
    }

    fn spawn(&self, executable: &str, inherit: &[Handle]) -> Result<Handle> {
        let inherit = inherit.iter().map(|handle| *handle);
        let process = process::spawn(String::from(executable), inherit)?;
        Ok(process::make_handle(process))
    }

    fn wait_for_exit(&self, process: Handle) -> Result<i32> {
        let deferred = process::resolve_handle(process, |kobj| kobj.deferred_i32())?;
        Ok(deferred.get())
    }

    fn create_shared_mem(&self) -> Result<Handle> {
        Ok(process::make_handle(Arc::new(SharedMemBlock::new())))
    }

    fn map_shared_mem(&self, block: Handle, len: usize, writable: bool) -> Result<*mut u8> {
        let block = process::resolve_handle_ref(block, |kobj| kobj.shared_mem_block())?;
        let slice = process::map_shared(block, len, true, writable)?;
        Ok(slice.as_mut_ptr())
    }

    fn create_pipe(&self) -> Result<Handle> {
        Ok(process::make_handle(Arc::new(Pipe::new())))
    }

    fn open_handle(&self, from_process: Handle, from_handle: usize) -> Result<Handle> {
        let from_process = process::resolve_handle_ref(from_process, |kobj| kobj.process())?;
        let from_handle = from_process.resolve_handle_ref(from_handle, |kobj| Some(kobj))?;
        Ok(process::make_handle(from_handle.get().clone()))
    }
}
