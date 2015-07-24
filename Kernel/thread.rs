use ::arch::cpu::{self,Regs};
use ::arch::debug;
use ::arch::keyboard::Keyboard;
use ::arch::thread;
use ::phys_mem::{self,PhysicalBitmap};
use ::process::Process;
use ::virt_mem::VirtualTree;
use alloc::heap;
use libc::{self,jmp_buf};
use spin::{Mutex,MutexGuard};
use std::boxed::FnBox;
use std::collections::VecDeque;
use std::mem;
use std::slice;
use std::slice::bytes;
use std::str;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

pub fn setjmp() -> Option<jmp_buf> {
    unsafe {
        let mut jmp_buf = mem::uninitialized();
        if libc::setjmp(&mut jmp_buf) == 0 {
            Some(jmp_buf)
        } else {
            None
        }
    }
}

fn drop_write_guard<'a, T>(guard: MutexGuard<'a, T>) {
    mem::drop(guard);
}

fn forget_write_guard<'a, T>(guard: MutexGuard<'a, T>) {
    mem::forget(guard);
}

fn assert_no_lock() {
    assert!(cpu::interrupts_enabled());
}

struct Thread {
    id: usize,
    stack: &'static mut [u8],
    process: Arc<Process>
}

impl Thread {
    pub fn new(process: Arc<Process>, stack: &'static mut [u8]) -> Thread {
        static NEXT_THREAD_ID: AtomicUsize = ATOMIC_USIZE_INIT;
        Thread {
            id: NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed),
            stack: stack,
            process: process
        }
    }
}

struct DeferredState<A> {
    result: Option<A>,
    waiters: VecDeque<(jmp_buf, Thread)>
}

struct SchedulerState {
    current: Thread,
    threads: VecDeque<(jmp_buf, Thread)>,
    garbage_stacks: Vec<&'static mut [u8]>
}

#[derive(Clone)]
pub struct Deferred<A> {
    scheduler: Arc<Scheduler>,
    state: Arc<Mutex<DeferredState<A>>>
}

pub struct Scheduler {
    state: Mutex<SchedulerState>
}

impl Scheduler {
    pub fn new(idle_process: Arc<Process>) -> Scheduler {
        let state = SchedulerState {
            current: Thread::new(idle_process, &mut []),
            threads: VecDeque::new(),
            garbage_stacks: Vec::new()
        };

        Scheduler { state: Mutex::new(state) }
    }

    pub fn schedule(&self) {
        assert_no_lock();

        let mut state = lock!(self.state);
        match state.threads.pop_front() {
            Some((new_jmp_buf, new_current)) => {
                let old_current = mem::replace(&mut state.current, new_current);

                match setjmp() {
                    Some(old_jmp_buf) => {
                        state.threads.push_back((old_jmp_buf, old_current));
                        drop_write_guard(state);

                        unsafe {
                            libc::longjmp(&new_jmp_buf, 1);
                        }
                    },

                    None => {
                        forget_write_guard(state);
                    }
                }
            },

            None => { }
        };
    }

    pub fn exit_current(&self) -> ! {
        assert_no_lock();

        let mut state = lock!(self.state);
        let (new_jmp_buf, new_current) =
            match state.threads.pop_front() {
                Some(front) => front,
                None => panic!("exit_current({}): no more threads", state.current.id)
            };

        let old_current = mem::replace(&mut state.current, new_current);
        state.garbage_stacks.push(old_current.stack);
        drop_write_guard(state);

        unsafe {
            libc::longjmp(&new_jmp_buf, 1);
        }
    }

    fn get_deferred<A>(&self, dstate: &Mutex<DeferredState<A>>) -> A {
        loop {
            assert_no_lock();

            let mut dstate = lock!(dstate);
            match mem::replace(&mut dstate.result, None) {
                Some(result) => { return result; },
                None => { }
            }

            let mut state = lock!(self.state);
            let (new_jmp_buf, new_current) =
                match state.threads.pop_front() {
                    Some(front) => front,
                    None => panic!("block({}): no more threads", state.current.id)
                };

            let old_current = mem::replace(&mut state.current, new_current);

            match setjmp() {
                Some(old_jmp_buf) => {
                    dstate.waiters.push_back((old_jmp_buf, old_current));
                    drop_write_guard(state);
                    drop_write_guard(dstate);

                    unsafe {
                        libc::longjmp(&new_jmp_buf, 1);
                    }
                },

                None => {
                    forget_write_guard(state);
                    forget_write_guard(dstate);
                }
            }
        }
    }

    fn try_get_deferred<A>(&self, dstate: &Mutex<DeferredState<A>>) -> Option<A> {
        mem::replace(&mut lock!(dstate).result, None)
    }

    fn resolve_deferred<A>(&self, dstate: &Mutex<DeferredState<A>>, result: A) {
        let mut dstate = lock!(dstate);
        match dstate.result {
            Some(_) => panic!("promise is already resolved"),
            None => { }
        }

        let mut state = lock!(self.state);
        dstate.result = Some(result);

        let mut waiters = mem::replace(&mut dstate.waiters, VecDeque::new());
        state.threads.append(&mut waiters);
    }

    fn spawn_inner(&self, process: Arc<Process>, start: Box<FnBox()>) {
        let stack_len = 4096;
        let stack = unsafe { slice::from_raw_parts_mut(heap::allocate(stack_len, 16), stack_len) };
        let jmp_buf = thread::new_jmp_buf(Box::new(start), stack);
        let thread = Thread::new(process, stack);

        {
            let mut state = lock!(self.state);
            state.threads.push_back((jmp_buf, thread));
        }
    }

    pub fn spawn_user_mode(&self, pc: *const u8, stack: &mut [u8]) {
        let stack_ptr = stack.as_mut_ptr();
        let stack_len = stack.len();

        let start = move || {
            unsafe {
                thread::jmp_user_mode(pc, stack_ptr.offset(stack_len as isize))
            }
            // TODO: free stack
        };

        let process = lock!(self.state).current.process.clone();
        self.spawn_inner(process, Box::new(start))
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        let garbage_stacks = {
            let mut state = lock!(self.state);
            mem::replace(&mut state.garbage_stacks, Vec::new())
        };

        for stack in garbage_stacks {
            unsafe { heap::deallocate(stack.as_mut_ptr(), stack.len(), 16) };
        }
    }
}

pub trait Promise<A> {
    fn get(&self) -> A;
    fn try_get(&self) -> Option<A>;
    // fn then<B>(f: FnOnce(A) -> B) -> Promise<B>;
}

impl<A> Deferred<A> {
    pub fn new(scheduler: Arc<Scheduler>) -> Deferred<A> {
        let dstate = Arc::new(Mutex::new(DeferredState {
            result: None,
            waiters: VecDeque::new()
        }));

        Deferred { scheduler: scheduler, state: dstate }
    }

    pub fn resolve(&self, result: A) {
        self.scheduler.resolve_deferred(&self.state, result)
    }
}

impl<A> Promise<A> for Deferred<A> {
    fn get(&self) -> A {
        self.scheduler.get_deferred(&self.state)
    }

    fn try_get(&self) -> Option<A> {
        self.scheduler.try_get_deferred(&self.state)
    }
}

pub fn spawn_remote<T, A>(scheduler: Arc<Scheduler>, process: Arc<Process>, start: T) -> Deferred<A> where T : FnOnce() -> A + 'static, A : 'static {
    let dstate = Arc::new(Mutex::new(DeferredState {
        result: None,
        waiters: VecDeque::new()
    }));

    let start = {
        let dstate = dstate.clone();
        let scheduler = scheduler.clone();
        move || {
            let result = start();
            scheduler.resolve_deferred(&dstate, result);
            scheduler.exit_current();
        }
    };

    scheduler.spawn_inner(process, Box::new(start));

    Deferred { scheduler: scheduler, state: dstate }
}

pub fn spawn<T, A>(scheduler: Arc<Scheduler>, start: T) -> Deferred<A> where T : FnOnce() -> A + 'static, A : 'static {
    let process = lock!(scheduler.state).current.process.clone();
    spawn_remote(scheduler, process, start)
}

test! {
    fn can_spawn_exit_thread() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Arc::new(Scheduler::new(p.clone()));
        let d = spawn(scheduler, || 123);
        assert_eq!(123, d.get());
    }

    fn can_spawn_exit_two_threads() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Arc::new(Scheduler::new(p.clone()));
        let d1 = spawn(scheduler.clone(), || 456);
        let d2 = spawn(scheduler, || 789);
        assert_eq!(456, d1.get());
        assert_eq!(789, d2.get());
    }

    fn can_closure() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Arc::new(Scheduler::new(p.clone()));
        let s = String::from_str("hello");
        let d = spawn(scheduler, move || s + &" world");
        assert_eq!("hello world", d.get());
    }

    fn threads_can_spawn_more_threads() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Arc::new(Scheduler::new(p.clone()));

        let thread2_fn = || 1234;

        let thread1_fn = {
            let scheduler = scheduler.clone();
            || {
                let d = spawn(scheduler, thread2_fn);
                d.get() * 2
            }
        };

        let d = spawn(scheduler, thread1_fn);
        assert_eq!(1234 * 2, d.get());
    }

    fn can_run_hello_world() {
        static HELLO: &'static [u8] = include_bytes!("../hello/hello.bin");
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::for_kernel());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Arc::new(Scheduler::new(p.clone()));
        let keyboard = Keyboard::new(scheduler.clone());
        p.switch();

        let mut code_slice = p.alloc(HELLO.len(), true, true).unwrap();
        let stack_slice = p.alloc(phys_mem::PAGE_SIZE, true, true).unwrap();
        log!("code_slice = {:p}, stack_slice = {:p}", code_slice.as_ptr(), stack_slice.as_ptr());
        bytes::copy_memory(HELLO, code_slice);
        log!("code_slice = {:?}", &code_slice[0..16]);

        let dstate = Arc::new(Mutex::new(DeferredState {
            result: None,
            waiters: VecDeque::new()
        }));

        let syscall_handler = {
            let dstate = dstate.clone();
            let scheduler = scheduler.clone();
            move |regs: &Regs| -> usize {
                match regs.rax {
                    0 => {
                        let bytes = unsafe { slice::from_raw_parts(regs.rdi as *const u8, regs.rsi as usize) };
                        debug::puts(str::from_utf8(bytes).unwrap());
                        0
                    },
                    1 => {
                        scheduler.resolve_deferred(&dstate, regs.rdi);
                        scheduler.exit_current();
                    },
                    2 => {
                        let bytes = unsafe { slice::from_raw_parts_mut(regs.rdi as *mut u8, regs.rsi as usize) };
                        keyboard.read_line(bytes)
                    },
                    _ => regs.rax as usize
                }
            }
        };

        let d = Deferred { scheduler: scheduler.clone(), state: dstate };
        let _x = thread::register_syscall_handler(syscall_handler);
        scheduler.spawn_user_mode(code_slice.as_ptr(), stack_slice);

        assert_eq!(0x1234, d.get());
    }
}
