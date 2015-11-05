use alloc::arc::Arc;
use arch::cpu;
use arch::keyboard::Keyboard;
use arch::vga::Vga;
use console::Console;
use deferred::Deferred;
use ksyscall::{self,SyscallHandler};
use prelude::*;
use process::{self,KObj,KObjRef};
use thread;

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
            let handler = SyscallHandler::new(
                Arc::new(Console::new(
                    KObjRef::new(Arc::new(Keyboard::new()), |kobj| kobj.async_read()).unwrap(),
                    KObjRef::new(Arc::new(Vga::new()), |kobj| kobj.write()).unwrap())));

            let _x = ksyscall::register_handler(handler);
            let (_, deferred) = process::spawn(String::from("hello")).unwrap();
            deferred.poll();
        });
    }
}
