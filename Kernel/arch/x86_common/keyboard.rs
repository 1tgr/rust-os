use ::arch::isr::{self,DropIrqHandler};
use ::phys_mem::PhysicalBitmap;
use ::process::Process;
use ::thread::{Deferred,Promise,Scheduler};
use ::virt_mem::VirtualTree;
use std::sync::Arc;

pub struct Keyboard {
    _drop_irq_handler: DropIrqHandler,
}

impl Keyboard {
    pub fn new<T>(handler: T) -> Keyboard where T : Fn() + 'static {
        Keyboard {
            _drop_irq_handler: isr::register_irq_handler(1, handler)
        }
    }
}

test! {
    fn can_read_key() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Arc::new(Scheduler::new(p));
        let d = Arc::new(Deferred::new(scheduler));

        let handler = {
            let d = d.clone();
            move || {
                log!("key pressed");
                d.resolve(());
            }
        };

        let _keyboard = Keyboard::new(handler);
        log!("Press any key to continue");
        while let None = d.try_get() {
            unsafe { asm!("hlt") };
        }
    }
}
