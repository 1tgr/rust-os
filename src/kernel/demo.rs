use crate::arch::cpu;
use crate::arch::keyboard::Keyboard;
use crate::arch::ps2_mouse::Ps2Mouse;
use crate::arch::vga::Vga;
use crate::deferred::Deferred;
use crate::ksyscall::{self, SyscallHandler};
use crate::process;
use crate::thread;
use alloc::sync::Arc;

impl<A> Deferred<A> {
    fn poll(mut self) -> A {
        loop {
            match self.try_get() {
                Ok(result) => return result,

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
            let handler = SyscallHandler::new(Arc::new(Ps2Mouse::new()));
            let _x = ksyscall::register_handler(handler);

            let stdin = Arc::new(Keyboard::new());
            let stdout = Arc::new(Vga::new());
            let process = process::spawn("graphics_server".into(), vec![Some(stdin), Some(stdout)]).unwrap();
            assert_eq!(0, process.exit_code().poll());
        });
    }
}
