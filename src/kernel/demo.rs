use ::arch::debug;
use ::arch::keyboard::Keyboard;
use ::arch::thread;
use ::phys_mem::{self,PhysicalBitmap};
use ::process::Process;
use ::thread::{Deferred,Promise,Scheduler};
use ::virt_mem::VirtualTree;
use std::slice::bytes;
use std::sync::Arc;
use syscall::{ErrNum,Handler};

struct TestSyscallHandler<'a> {
    scheduler: &'a Scheduler,
    deferred: Arc<Deferred<'a, i32>>,
    keyboard: Keyboard<'a>,
    process: Arc<Process>
}

impl<'a> Handler for TestSyscallHandler<'a> {
    fn write(&self, s: &str) -> Result<(), ErrNum> {
        Ok(debug::puts(s))
    }

    fn exit_thread(&self, code: i32) -> Result<(), ErrNum> {
        self.deferred.resolve(code);
        self.scheduler.exit_current()
    }

    fn read_line(&self, buf: &mut [u8]) -> Result<usize, ErrNum> {
        Ok(self.keyboard.read_line(buf))
    }

    fn alloc_pages(&self, len: usize) -> Result<*mut u8, ErrNum> {
        match self.process.alloc(len, true, true) {
            Ok(slice) => Ok(slice.as_mut_ptr()),
            Err(_) => Err(ErrNum::OutOfMemory)
        }
    }

    fn free_pages(&self, p: *mut u8) -> Result<bool, ErrNum> {
        Ok(self.process.free(p))
    }
}

test! {
    fn can_run_hello_world() {
        static HELLO: &'static [u8] = include_bytes!("../hello/hello.bin");
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::for_kernel());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Scheduler::new(p.clone());
        let keyboard = Keyboard::new(&scheduler);
        p.switch();

        let mut code_slice = p.alloc(HELLO.len(), true, true).unwrap();
        let stack_slice = p.alloc(phys_mem::PAGE_SIZE, true, true).unwrap();
        log!("code_slice = {:p}, stack_slice = {:p}", code_slice.as_ptr(), stack_slice.as_ptr());
        bytes::copy_memory(HELLO, code_slice);
        log!("code_slice = {:?}", &code_slice[0..16]);

        let deferred = Arc::new(Deferred::new(&scheduler));

        let handler = TestSyscallHandler {
            scheduler: &scheduler,
            deferred: deferred.clone(),
            keyboard: keyboard,
            process: p.clone()
        };

        let _x = thread::register_syscall_handler(handler);
        scheduler.spawn_user_mode(code_slice.as_ptr(), stack_slice);

        assert_eq!(0x1234, deferred.get());
    }
}
