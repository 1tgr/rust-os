use alloc::arc::Arc;
use alloc::boxed::FnBox;
use alloc::heap;
use arch::cpu;
use arch::thread;
use collections::vec_deque::VecDeque;
use core::mem;
use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use libc::{self,jmp_buf};
use mutex::{Mutex,MutexGuard};
use phys_mem::PhysicalBitmap;
use prelude::*;
use process::Process;
use singleton::Singleton;
use virt_mem::VirtualTree;

static SCHEDULER: Singleton<Mutex<SchedulerState>> = Singleton::new();

fn setjmp() -> Option<jmp_buf> {
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
    process: Arc<Process>,
    exited: Deferred<i32>
}

impl Thread {
    pub fn new(process: Arc<Process>, stack: &'static mut [u8]) -> Thread {
        static NEXT_THREAD_ID: AtomicUsize = ATOMIC_USIZE_INIT;
        Thread {
            id: NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed),
            stack: stack,
            process: process,
            exited: Deferred::new()
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

impl Drop for SchedulerState {
    fn drop(&mut self) {
        let garbage_stacks = mem::replace(&mut self.garbage_stacks, Vec::new());

        for stack in garbage_stacks {
            unsafe { heap::deallocate(stack.as_mut_ptr(), stack.len(), 16) };
        }
    }
}

pub struct Deferred<A> {
    state: Arc<Mutex<DeferredState<A>>>
}

pub fn with_scheduler<F: FnOnce()>(f: F) {
    let phys = Arc::new(PhysicalBitmap::parse_multiboot());
    let kernel_virt = Arc::new(VirtualTree::for_kernel());
    let idle_process = Arc::new(Process::new(phys, kernel_virt).unwrap());

    let state = SchedulerState {
        current: Thread::new(idle_process.clone(), &mut []),
        threads: VecDeque::new(),
        garbage_stacks: Vec::new()
    };

    let _d = SCHEDULER.register(Mutex::new(state));
    unsafe { idle_process.switch() };
    f()
}

fn current_sched() -> &'static Mutex<SchedulerState> {
    SCHEDULER.get().expect("no Scheduler registered")
}

macro_rules! lock_sched {
    () => (lock!(current_sched()))
}

pub fn current_process() -> Arc<Process> {
    lock_sched!().current.process.clone()
}

pub fn schedule() {
    assert_no_lock();

    let mut state = lock_sched!();
    match state.threads.pop_front() {
        Some((new_jmp_buf, new_current)) => {
            let new_process = new_current.process.clone();
            let old_current = mem::replace(&mut state.current, new_current);

            match setjmp() {
                Some(old_jmp_buf) => {
                    state.threads.push_back((old_jmp_buf, old_current));
                    drop_write_guard(state);

                    unsafe {
                        new_process.switch();
                        mem::drop(new_process);
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

pub fn exit(code: i32) -> ! {
    let exited = lock_sched!().current.exited.clone();
    exited.resolve(code);

    let new_jmp_buf = {
        assert_no_lock();

        let mut state = lock_sched!();
        let (new_jmp_buf, new_current) =
            match state.threads.pop_front() {
                Some(front) => front,
                None => panic!("exit({}): no more threads", state.current.id)
            };

        let new_process = new_current.process.clone();
        let old_current = mem::replace(&mut state.current, new_current);
        state.garbage_stacks.push(old_current.stack);
        unsafe { new_process.switch() };
        new_jmp_buf
    };

    unsafe {
        libc::longjmp(&new_jmp_buf, 1);
    }
}

fn spawn_inner<'a>(process: Arc<Process>, start: Box<FnBox() + 'a>) -> Deferred<i32> {
    let stack_len = 4096;
    let stack_base_ptr = unsafe { heap::allocate(stack_len, 16) };
    let b: Box<FnBox() + 'a> = Box::new(start);
    let jmp_buf = thread::new_jmp_buf(b, unsafe { stack_base_ptr.offset(stack_len as isize) });
    let thread = Thread::new(process, unsafe { slice::from_raw_parts_mut(stack_base_ptr, stack_len) });
    let exited = thread.exited.clone();

    {
        let mut state = lock_sched!();
        state.threads.push_back((jmp_buf, thread));
    }

    exited
}

pub fn spawn_user_mode(pc: *const u8, stack: &mut [u8]) -> Deferred<i32> {
    let stack_ptr = stack.as_mut_ptr();
    let stack_len = stack.len();

    let start = move || {
        unsafe {
            thread::jmp_user_mode(pc, stack_ptr.offset(stack_len as isize))
        }
        // TODO: free stack
    };

    let process = lock_sched!().current.process.clone();
    spawn_inner(process, Box::new(start))
}

pub fn spawn_remote<T>(process: Arc<Process>, start: T) -> Deferred<i32> where T : FnOnce() -> i32 {
    let start = || { exit(start()) };
    spawn_inner(process, Box::new(start))
}

pub fn spawn<T>(start: T) -> Deferred<i32> where T : FnOnce() -> i32 {
    let process = lock_sched!().current.process.clone();
    spawn_remote(process, start)
}

impl<A> Clone for Deferred<A> {
    fn clone(&self) -> Self {
        Deferred { state: self.state.clone() }
    }
}

impl<A> Deferred<A> {
    pub fn new() -> Self {
        let dstate = Arc::new(Mutex::new(DeferredState {
            result: None,
            waiters: VecDeque::new()
        }));

        Deferred { state: dstate }
    }

    pub fn resolve(&self, result: A) {
        let mut dstate = lock!(self.state);
        match dstate.result {
            Some(_) => panic!("promise is already resolved"),
            None => { }
        }

        dstate.result = Some(result);

        let mut waiters = mem::replace(&mut dstate.waiters, VecDeque::new());
        if waiters.len() > 0 {
            let mut state = lock_sched!();
            state.threads.append(&mut waiters);
        }
    }

    pub fn get(self) -> A {
        loop {
            assert_no_lock();

            let mut dstate = lock!(self.state);
            match mem::replace(&mut dstate.result, None) {
                Some(result) => { return result; },
                None => { }
            }

            let mut state = lock_sched!();
            let (new_jmp_buf, new_current) =
                match state.threads.pop_front() {
                    Some(front) => front,
                    None => panic!("block({}): no more threads", state.current.id)
                };

            let new_process = new_current.process.clone();
            let old_current = mem::replace(&mut state.current, new_current);

            match setjmp() {
                Some(old_jmp_buf) => {
                    dstate.waiters.push_back((old_jmp_buf, old_current));
                    drop_write_guard(state);
                    drop_write_guard(dstate);

                    unsafe {
                        new_process.switch();
                        mem::drop(new_process);
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

    pub fn try_get(self) -> Result<A, Self> {
        let opt = {
            let mut state = lock!(self.state);
            mem::replace(&mut state.result, None)
        };

        opt.ok_or(self)
    }
}

#[cfg(feature = "test")]
pub mod test {
    use prelude::*;
    use super::*;

    test! {
        fn can_spawn_exit_thread() {
            with_scheduler(|| {
                let d = spawn(|| 123);
                assert_eq!(123, d.get());
            });
        }

        fn can_spawn_exit_two_threads() {
            with_scheduler(|| {
                let d1 = spawn(|| 456);
                let d2 = spawn(|| 789);
                assert_eq!(456, d1.get());
                assert_eq!(789, d2.get());
            });
        }

        fn can_closure() {
            with_scheduler(|| {
                let s = String::from("hello");
                let d = spawn(move || (s + &" world").len() as i32);
                assert_eq!(11, d.get());
            });
        }

        fn threads_can_spawn_more_threads() {
            with_scheduler(|| {
                let thread2_fn = || 1234;

                let thread1_fn = {
                    || {
                        let d = spawn(thread2_fn);
                        d.get() * 2
                    }
                };

                let d = spawn(thread1_fn);
                assert_eq!(1234 * 2, d.get());
            });
        }
    }
}
