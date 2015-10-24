use alloc::arc::Arc;
use arch::keyboard::Keyboard;
use arch::vga_bochs;
use arch::vga::Vga;
use console::Console;
use io::{Read,Write};
use phys_mem::PhysicalBitmap;
use process::{self,KObj,Process};
use syscall::{ErrNum,Handle,FileHandle,Handler,Result};
use thread;
use virt_mem::VirtualTree;

struct TestSyscallHandler {
    console: Arc<Console>
}

impl Handler for TestSyscallHandler {
    fn exit_thread(&self, code: i32) -> Result<()> {
        thread::exit(code)
    }

    fn write(&self, file: FileHandle, bytes: &[u8]) -> Result<usize> {
        let kobj = try!(thread::current_process().resolve_handle(file).ok_or(ErrNum::InvalidHandle));
        let file: &Write = try!(kobj.write().ok_or(ErrNum::NotSupported));
        file.write(bytes)
    }

    fn read(&self, file: FileHandle, buf: &mut [u8]) -> Result<usize> {
        let kobj = try!(thread::current_process().resolve_handle(file).ok_or(ErrNum::InvalidHandle));
        let file: &Read = try!(kobj.read().ok_or(ErrNum::NotSupported));
        file.read(buf)
    }

    fn alloc_pages(&self, len: usize) -> Result<*mut u8> {
        match thread::current_process().alloc(len, true, true) {
            Ok(slice) => Ok(slice.as_mut_ptr()),
            Err(_) => Err(ErrNum::OutOfMemory)
        }
    }

    fn free_pages(&self, p: *mut u8) -> Result<bool> {
        Ok(thread::current_process().free(p))
    }

    fn open(&self, filename: &str) -> Result<FileHandle> {
        let file: Arc<KObj> =
            match filename {
                "stdin" => self.console.clone(),
                "stdout" => self.console.clone(),
                _ => { return Err(ErrNum::FileNotFound) }
            };

        Ok(thread::current_process().make_handle(file))
    }

    fn close(&self, handle: Handle) -> Result<()> {
        if !thread::current_process().close_handle(handle) {
            return Err(ErrNum::InvalidHandle)
        }

        Ok(())
    }

    fn init_video_mode(&self, width: u16, height: u16, bpp: u8) -> Result<*mut u8> {
        let process = thread::current_process();
        let slice = try!(vga_bochs::init(&*process, width, height, bpp));
        Ok(slice.as_mut_ptr())
    }
}

test! {
    fn can_run_hello_world() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::for_kernel());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        thread::with_scheduler(p.clone(), || {
            let handler = TestSyscallHandler {
                console: Arc::new(Console::new(Arc::new(Keyboard::new()), Arc::new(Vga::new())))
            };

            let _x = ::arch::thread::register_syscall_handler(handler);
            let (_, mut deferred) = process::spawn("hello").unwrap();

            let code;
            loop {
                match deferred.try_get() {
                    Ok(n) => {
                        code = n;
                        break;
                    },
                    Err(d) => {
                        deferred = d;
                        thread::schedule();
                    }
                }
            }

            assert_eq!(0x1234, code);
        });
    }
}
