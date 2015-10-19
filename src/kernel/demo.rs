use alloc::arc::Arc;
use arch::keyboard::Keyboard;
use arch::vga::Vga;
use console::Console;
use core::mem;
use core::slice;
use io::{Read,Write};
use miniz_sys as mz;
use multiboot::multiboot_module_t;
use phys_mem::{self,PhysicalBitmap};
use process::{KObj,Process};
use syscall::{ErrNum,Handle,FileHandle,Handler,Result};
use thread::{self,Deferred};
use virt_mem::VirtualTree;

struct TestSyscallHandler {
    console: Arc<Console>,
    deferred: Deferred<i32>,
    process: Arc<Process>
}

impl Handler for TestSyscallHandler {
    fn exit_thread(&self, code: i32) -> Result<()> {
        self.deferred.resolve(code);
        thread::exit()
    }

    fn write(&self, file: FileHandle, bytes: &[u8]) -> Result<usize> {
        let kobj = try!(self.process.resolve_handle(file).ok_or(ErrNum::InvalidHandle));
        let file: &Write = try!(kobj.write().ok_or(ErrNum::NotSupported));
        file.write(bytes)
    }

    fn read(&self, file: FileHandle, buf: &mut [u8]) -> Result<usize> {
        let kobj = try!(self.process.resolve_handle(file).ok_or(ErrNum::InvalidHandle));
        let file: &Read = try!(kobj.read().ok_or(ErrNum::NotSupported));
        file.read(buf)
    }

    fn alloc_pages(&self, len: usize) -> Result<*mut u8> {
        match self.process.alloc(len, true, true) {
            Ok(slice) => Ok(slice.as_mut_ptr()),
            Err(_) => Err(ErrNum::OutOfMemory)
        }
    }

    fn free_pages(&self, p: *mut u8) -> Result<bool> {
        Ok(self.process.free(p))
    }

    fn open(&self, filename: &str) -> Result<FileHandle> {
        let file: Arc<KObj> =
            match filename {
                "stdin" => self.console.clone(),
                "stdout" => self.console.clone(),
                _ => { return Err(ErrNum::FileNotFound) }
            };

        Ok(self.process.make_handle(file))
    }

    fn close(&self, handle: Handle) -> Result<()> {
        if !self.process.close_handle(handle) {
            return Err(ErrNum::InvalidHandle)
        }

        Ok(())
    }
}

test! {
    fn can_run_hello_world() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::for_kernel());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        thread::with_scheduler(p.clone(), || {
            p.switch();

            let code_slice = p.alloc(2 * 1024 * 1024, true, true).unwrap();
            unsafe {
                let info = phys_mem::multiboot_info();

                let mods: &[multiboot_module_t] = slice::from_raw_parts(
                    phys_mem::phys2virt(info.mods_addr as usize),
                    info.mods_count as usize);

                assert_eq!(1, mods.len());

                let mod_data: &[u8] = slice::from_raw_parts(
                    phys_mem::phys2virt(mods[0].mod_start as usize),
                    (mods[0].mod_end - mods[0].mod_start) as usize);

                assert_eq!(('P', 'K'), (mod_data[0] as char, mod_data[1] as char));

                let mut zip = mem::zeroed();
                assert!(mz::mz_zip_reader_init_mem(&mut zip, mod_data.as_ptr() as *const _, mod_data.len() as u64, 0));

                let index = mz::mz_zip_reader_locate_file(&mut zip, b"hello.bin\0" as *const _, 0 as *const _, 0);
                assert!(index >= 0);

                let index = index as u32;
                let mut stat = mem::zeroed();
                assert!(mz::mz_zip_reader_file_stat(&mut zip, index, &mut stat));
                assert!(stat.uncomp_size as usize <= code_slice.len());
                assert!(mz::mz_zip_reader_extract_to_mem(&mut zip, index, code_slice.as_mut_ptr() as *mut _, stat.uncomp_size as u64, 0));
                mz::mz_zip_reader_end(&mut zip);
            }

            let stack_slice = p.alloc(phys_mem::PAGE_SIZE, true, true).unwrap();
            log!("code_slice = {:p}, stack_slice = {:p}", code_slice.as_ptr(), stack_slice.as_ptr());
            log!("code_slice = {:?}", &code_slice[0..16]);

            let mut deferred = Deferred::new();

            let handler = TestSyscallHandler {
                console: Arc::new(Console::new(Arc::new(Keyboard::new()), Arc::new(Vga::new()))),
                deferred: deferred.clone(),
                process: p.clone()
            };

            let _x = ::arch::thread::register_syscall_handler(handler);
            thread::spawn_user_mode(code_slice.as_ptr(), stack_slice);

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
