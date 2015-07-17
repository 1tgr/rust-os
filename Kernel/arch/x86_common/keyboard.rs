use ::arch::isr::{self,DropIrqHandler};
use ::phys_mem::PhysicalBitmap;
use ::arch::cpu;
use ::process::Process;
use ::thread::{Deferred,Promise,Scheduler};
use ::virt_mem::VirtualTree;
use spin::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

pub trait Device {
    fn read_async(&self, buf: Vec<u8>) -> Deferred<Result<(Vec<u8>, usize), &'static str>>;
}

pub struct IoRequest {
    buf: Vec<u8>,
    d: Deferred<Result<(Vec<u8>, usize), &'static str>>,
    current: usize
}

impl IoRequest {
    pub fn new(buf: Vec<u8>, d: Deferred<Result<(Vec<u8>, usize), &'static str>>) -> IoRequest {
        IoRequest {
            buf: buf,
            d: d,
            current: 0
        }
    }

    pub fn fulfil(mut self, b: u8) -> Option<IoRequest> {
        self.buf[self.current] = b;
        self.current += 1;
        if self.current >= self.buf.len() {
            self.d.resolve(Ok((self.buf, self.current)));
            None
        } else {
            Some(self)
        }
    }
}

pub struct Keyboard {
    _drop_irq_handler: DropIrqHandler,
    scheduler: Arc<Scheduler>,
    requests: Arc<Mutex<VecDeque<IoRequest>>>
}

impl Keyboard {
    pub fn new(scheduler: Arc<Scheduler>) -> Keyboard {
        let requests = Arc::new(Mutex::new(VecDeque::new()));

        let handler = {
            let requests = requests.clone();
            move || {
                let (b, request_opt): (u8, Option<IoRequest>) = {
                    let mut requests = lock!(requests);
                    (unsafe { cpu::inb(0x60) }, requests.pop_front())
                };

                if let Some(request) = request_opt {
                    if let Some(request) = request.fulfil(b) {
                        lock!(requests).push_front(request);
                    }
                }
            }
        };

        Keyboard {
            _drop_irq_handler: isr::register_irq_handler(1, handler),
            scheduler: scheduler,
            requests: requests
        }
    }
}

impl<'a> Device for Keyboard {
    fn read_async(&self, buf: Vec<u8>) -> Deferred<Result<(Vec<u8>, usize), &'static str>> {
        let d = Deferred::new(self.scheduler.clone());
        lock!(self.requests).push_back(IoRequest::new(buf, d.clone()));
        d
    }
}

test! {
    fn can_read_key() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Arc::new(Scheduler::new(p));
        let keyboard = Keyboard::new(scheduler);
        let d = keyboard.read_async(vec![0; 1]);
        log!("Press any key to continue");

        loop {
            if let Some(result) = d.try_get() {
                let (keys, _) = result.unwrap(); 
                let k = keys.get(0).expect("");
                log!("You pressed: {}", k);
                break;
            } else {
                unsafe { asm!("hlt") };
            }
        }
    }
}
