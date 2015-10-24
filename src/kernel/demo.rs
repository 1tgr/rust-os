use alloc::arc::Arc;
use arch::cpu;
use arch::keyboard::Keyboard;
use arch::vga_bochs;
use arch::vga::Vga;
use console::Console;
use io::{Read,Write};
use process::{self,KObj};
use syscall::{ErrNum,Handle,FileHandle,Handler,Result};
use thread::{self,Deferred};

struct TestSyscallHandler {
    console: Arc<Console>
}

impl Handler for TestSyscallHandler {
    fn exit_thread(&self, code: i32) -> Result<()> {
        thread::exit(code)
    }

    fn write(&self, file: FileHandle, bytes: &[u8]) -> Result<usize> {
        let kobj = try!(process::resolve_handle(file).ok_or(ErrNum::InvalidHandle));
        let file: &Write = try!(kobj.write().ok_or(ErrNum::NotSupported));
        file.write(bytes)
    }

    fn read(&self, file: FileHandle, buf: &mut [u8]) -> Result<usize> {
        let kobj = try!(process::resolve_handle(file).ok_or(ErrNum::InvalidHandle));
        let file: &Read = try!(kobj.read().ok_or(ErrNum::NotSupported));
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

    fn open(&self, filename: &str) -> Result<FileHandle> {
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
}

impl<A> Deferred<A> {
    fn poll(mut self) -> A {
        loop {
            match self.try_get() {
                Ok(result) => {
                    return result
                },

                Err(d) => {
                    thread::schedule();
                    assert!(cpu::interrupts_enabled());
                    cpu::wait_for_interrupt();
                    self = d;
                }
            }
        }
    }
}

test! {
    fn can_run_hello_world() {
        thread::with_scheduler(|| {
            let handler = TestSyscallHandler {
                console: Arc::new(Console::new(Arc::new(Keyboard::new()), Arc::new(Vga::new())))
            };

            let _x = ::arch::thread::register_syscall_handler(handler);
            let (_, deferred) = process::spawn("hello").unwrap();
            assert_eq!(0x1234, deferred.poll());
        });
    }
}
