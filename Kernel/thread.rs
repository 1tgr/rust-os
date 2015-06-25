use ::arch::thread;
use ::phys_mem::PhysicalBitmap;
use ::process::Process;
use ::virt_mem::VirtualTree;
use alloc::heap;
use libc::{self,jmp_buf};
use spin::{RwLock,RwLockWriteGuard};
use std::boxed::FnBox;
use std::collections::VecDeque;
use std::mem;
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

fn drop_write_guard<'a, T>(guard: RwLockWriteGuard<'a, T>) {
    mem::drop(guard);
}

fn forget_write_guard<'a, T>(guard: RwLockWriteGuard<'a, T>) {
    mem::forget(guard);
}

static NEXT_THREAD_ID: AtomicUsize = ATOMIC_USIZE_INIT;

struct Thread {
    id: usize,
    stack: *mut u8,
    stack_len: usize,
    process: Arc<Process>
}

struct DeferredState<A> {
    result: Option<A>,
    waiters: VecDeque<(jmp_buf, Thread)>
}

struct SchedulerState {
    current: Thread,
    threads: VecDeque<(jmp_buf, Thread)>,
    garbage_stacks: Vec<(*mut u8, usize)>
}

pub struct Deferred<'a, A> {
    scheduler: &'a Scheduler,
    state: Arc<RwLock<DeferredState<A>>>
}

pub struct Scheduler {
    state: RwLock<SchedulerState>
}

impl Scheduler {
    pub fn new(idle_process: Arc<Process>) -> Scheduler {
        let idle = Thread {
            id: NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed),
            stack: 0 as *mut u8,
            stack_len: 0,
            process: idle_process
        };

        let state = SchedulerState {
            current: idle,
            threads: VecDeque::new(),
            garbage_stacks: Vec::new()
        };

        Scheduler { state: RwLock::new(state) }
    }

    pub fn schedule(&self) {
        let mut state = self.state.write();
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
        let mut state = self.state.write();
        let (new_jmp_buf, new_current) =
            match state.threads.pop_front() {
                Some(front) => front,
                None => panic!("exit_current({}): no more threads", state.current.id)
            };

        let old_current = mem::replace(&mut state.current, new_current);
        state.garbage_stacks.push((old_current.stack, old_current.stack_len));
        drop_write_guard(state);

        unsafe {
            libc::longjmp(&new_jmp_buf, 1);
        }
    }

    fn get_deferred<A>(&self, dstate: &RwLock<DeferredState<A>>) -> A where A : Clone {
        loop {
            let mut dstate = dstate.write();
            match dstate.result {
                Some(ref result) => { return result.clone(); },
                None => { }
            }

            let mut state = self.state.write();
            let (new_jmp_buf, new_current) =
                match state.threads.pop_front() {
                    Some(front) => front,
                    None => panic!("block({}: no more threads", state.current.id)
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

    fn resolve_deferred<A>(&self, dstate: &RwLock<DeferredState<A>>, result: A) {
        let mut dstate = dstate.write();
        match dstate.result {
            Some(_) => panic!("promise is already resolved"),
            None => { }
        }

        let mut state = self.state.write();
        dstate.result = Some(result);

        let mut waiters = mem::replace(&mut dstate.waiters, VecDeque::new());
        state.threads.append(&mut waiters);
    }

    fn spawn_inner<'a>(&'a self, process: Arc<Process>, start: Box<FnBox() + 'a>) {
        let stack_len = 4096;
        let stack = unsafe { heap::allocate(stack_len, 16) };
        let jmp_buf = thread::new_jmp_buf(Box::new(start), stack, stack_len);

        let thread = Thread {
            id: NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed),
            stack: stack,
            stack_len: stack_len,
            process: process
        };

        {
            let mut state = self.state.write();
            state.threads.push_back((jmp_buf, thread));
        }
    }

    pub fn spawn_remote<T, A>(&self, process: Arc<Process>, start: T) -> Deferred<A> where T : FnOnce() -> A, A : Clone {
        let dstate = Arc::new(RwLock::new(DeferredState {
            result: None,
            waiters: VecDeque::new()
        }));

        let dstate1 = dstate.clone();
        let dstate2 = dstate.clone();

        let start = move || {
            let result = start();
            self.resolve_deferred(&dstate1, result);
            self.exit_current();
        };

        self.spawn_inner(process, Box::new(start));

        Deferred { scheduler: &self, state: dstate2 }
    }

    pub fn spawn<T, A>(&self, start: T) -> Deferred<A> where T : FnOnce() -> A, A : Clone {
        let process = self.state.read().current.process.clone();
        self.spawn_remote(process, start)
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        let garbage_stacks = {
            let mut state = self.state.write();
            mem::replace(&mut state.garbage_stacks, Vec::new())
        };

        for (stack, stack_len) in garbage_stacks {
            unsafe {
                heap::deallocate(stack, stack_len, 16);
            }
        }
    }
}

pub trait Promise<A> {
    fn get(&self) -> A;
    // fn then<B>(f: FnOnce(A) -> B) -> Promise<B>;
}

impl<'a, A> Deferred<'a, A> {
    pub fn resolve(&self, result: A) {
        self.scheduler.resolve_deferred(&self.state, result)
    }
}

impl<'a, A> Promise<A> for Deferred<'a, A> where A : Clone {
    fn get(&self) -> A {
        self.scheduler.get_deferred(&self.state)
    }
}

test! {
    fn can_spawn_exit_thread() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Scheduler::new(p.clone());
        let d = scheduler.spawn(|| 123);
        assert_eq!(123, d.get());
    }

    fn can_spawn_exit_two_threads() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Scheduler::new(p.clone());
        let d1 = scheduler.spawn(|| 456);
        let d2 = scheduler.spawn(|| 789);
        assert_eq!(456, d1.get());
        assert_eq!(789, d2.get());
    }

    fn can_closure() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Scheduler::new(p.clone());
        let s = String::from_str("hello");
        let d = scheduler.spawn(move || s + &" world");
        assert_eq!("hello world", d.get());
    }

    fn threads_can_spawn_more_threads() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Scheduler::new(p.clone());

        let thread2_fn = || 1234;

        let thread1_fn = || {
            let d = scheduler.spawn(thread2_fn);
            d.get() * 2
        };

        let d = scheduler.spawn(thread1_fn);
        assert_eq!(1234 * 2, d.get());
    }
}
