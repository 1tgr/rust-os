use alloc::arc::Arc;
use arch::keyboard::Keyboard;
use arch::vga_bochs;
use arch::vga::Vga;
use console::Console;
use core::slice;
use core::slice::bytes;
use core::str;
use elf::*;
use io::{Read,Write};
use multiboot::multiboot_module_t;
use phys_mem::{self,PhysicalBitmap};
use prelude::*;
use process::{KObj,Process};
use ptr::Align;
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

    fn init_video_mode(&self, width: u16, height: u16, bpp: u8) -> Result<*mut u8> {
        let slice = try!(vga_bochs::init(&*self.process, width, height, bpp));
        Ok(slice.as_mut_ptr())
    }
}

#[repr(C)]
pub struct TarHeader {
    pub filename: [u8; 100],
    pub mode: [u8; 8],
    pub uid: [u8; 8],
    pub gid: [u8; 8],
    pub size: [u8; 12],
    pub mtime: [u8; 12],
    pub chksum: [u8; 8],
    pub typeflag: [u8; 1],
}

impl TarHeader {
    pub fn parse_size(&self) -> usize {
        let mut size = 0usize;
        let mut j = 11;
        let mut count = 1;

        while j > 0 {
            size += (self.size[j - 1] - ('0' as u8)) as usize * count;
            j -= 1;
            count *= 8;
        }

        size
    }
}

fn nul_terminate(s: &[u8]) -> &[u8] {
    match s.iter().position(|b| *b == 0) {
        Some(index) => &s[0..index],
        None => s
    }
}

fn tar_locate<'a>(data: &'a [u8], filename: &str) -> Option<&'a [u8]> {
    unsafe {
        let mut ptr = data.as_ptr();
        let end = ptr.offset(data.len() as isize);
        while ptr < end {
            let header = &*(ptr as *const TarHeader);
            let size = header.parse_size();
            let header_filename = nul_terminate(&header.filename[..]);
            ptr = ptr.offset(512);

            if let Ok(header_filename) = str::from_utf8(header_filename) {
                if header_filename == filename {
                    return Some(slice::from_raw_parts(ptr, size));
                }
            }

            ptr = ptr.offset(Align::up(size, 512) as isize);
        }

        None
    }
}

test! {
    fn can_run_hello_world() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::for_kernel());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        thread::with_scheduler(p.clone(), || {
            p.switch();

            let temp_slice = unsafe {
                let info = phys_mem::multiboot_info();

                let mods: &[multiboot_module_t] = slice::from_raw_parts(
                    phys_mem::phys2virt(info.mods_addr as usize),
                    info.mods_count as usize);

                assert_eq!(1, mods.len());

                let mod_data: &[u8] = slice::from_raw_parts(
                    phys_mem::phys2virt(mods[0].mod_start as usize),
                    (mods[0].mod_end - mods[0].mod_start) as usize);

                tar_locate(mod_data, "hello").expect("file 'hello' not found in initrd")
            };

            let ehdr = unsafe { *(temp_slice.as_ptr() as *const Elf64_Ehdr) };
            assert_eq!([ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3, ELFCLASS64, ELFDATA2LSB, EV_CURRENT], &ehdr.e_ident[0..7]);
            assert_eq!((ET_EXEC, EM_X86_64), (ehdr.e_type, ehdr.e_machine));

            let entry = ehdr.e_entry as *const u8;
            log!("entry point is {:p}", entry);

            let mut slices = Vec::new();
            for i in 0..ehdr.e_phnum {
                let phdr_offset = ehdr.e_phoff as isize + (i as isize) * (ehdr.e_phentsize as isize);
                let phdr = unsafe { *(temp_slice.as_ptr().offset(phdr_offset) as *const Elf64_Phdr) };
                if phdr.p_type != PT_LOAD {
                    continue;
                }

                log!("segment {}: {:x} bytes @ {:p} (file: {:x} bytes @ {:x})", i, phdr.p_memsz, phdr.p_vaddr as *mut u8, phdr.p_filesz, phdr.p_offset);
                assert!(phdr.p_memsz >= phdr.p_filesz);
                let slice = unsafe { p.alloc_at::<u8>(phdr.p_vaddr as *mut u8, phdr.p_memsz as usize, true, true).unwrap() };
                let file_slice = &temp_slice[phdr.p_offset as usize .. (phdr.p_offset + phdr.p_filesz) as usize];
                bytes::copy_memory(file_slice, slice);
                slices.push(slice);
            }

            assert!(slices.iter().any(|ref slice| {
                let slice_end = unsafe { slice.as_ptr().offset(slice.len() as isize) };
                entry >= slice.as_ptr() && entry < slice_end
            }));

            let stack_slice = p.alloc(phys_mem::PAGE_SIZE * 10, true, true).unwrap();
            log!("stack_slice = 0x{:x} bytes @ {:p}", stack_slice.len(), stack_slice.as_ptr());

            let mut deferred = Deferred::new();

            let handler = TestSyscallHandler {
                console: Arc::new(Console::new(Arc::new(Keyboard::new()), Arc::new(Vga::new()))),
                deferred: deferred.clone(),
                process: p.clone()
            };

            let _x = ::arch::thread::register_syscall_handler(handler);
            thread::spawn_user_mode(entry, stack_slice);

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
