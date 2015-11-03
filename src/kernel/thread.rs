use alloc::arc::Arc;
use alloc::boxed::FnBox;
use alloc::heap;
use arch::cpu;
use arch::isr;
use arch::thread;
use collections::vec_deque::VecDeque;
use core::mem;
use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use deferred::Deferred;
use libc::{self,jmp_buf};
use mutex::Mutex;
use prelude::*;
use process::Process;
use ptr::Align;
use singleton::Singleton;

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

struct HWToken {
    process: Arc<Process>,
    stack_ptr: *mut u8
}

impl HWToken {
    pub unsafe fn switch(self) {
        self.process.switch();
        isr::set_kernel_stack(self.stack_ptr);
    }
}

struct Thread {
    _id: usize,
    stack: &'static mut [u8],
    process: Arc<Process>,
    exited: Deferred<i32>
}

impl Thread {
    pub fn new(process: Arc<Process>, stack: &'static mut [u8]) -> Thread {
        static NEXT_THREAD_ID: AtomicUsize = ATOMIC_USIZE_INIT;
        Thread {
            _id: NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed),
            stack: stack,
            process: process,
            exited: Deferred::new()
        }
    }

    pub fn hw_token(&mut self) -> HWToken {
        HWToken {
            process: self.process.clone(),
            stack_ptr: Align::down(unsafe { self.stack.as_mut_ptr().offset(self.stack.len() as isize) }, 16)
        }
    }
}

pub struct BlockedThread(jmp_buf, Thread);

struct SchedulerState {
    current: Thread,
    threads: VecDeque<BlockedThread>,
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

pub fn with_scheduler<F: FnOnce()>(f: F) {
    let idle_process = Arc::new(Process::for_kernel().unwrap());

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

pub fn block<Park: FnOnce(BlockedThread)>(park: Park) -> bool {
    let mut state = lock_sched!();
    match state.threads.pop_front() {
        Some(BlockedThread(new_jmp_buf, mut new_current)) => {
            let switch = {
                let new_token = new_current.hw_token();
                let old_current = mem::replace(&mut state.current, new_current);
                mem::drop(state);

                move |old_jmp_buf| {
                    park(BlockedThread(old_jmp_buf, old_current));
                    assert!(cpu::interrupts_enabled(), "current thread is holding a spinlock and it's about to be blocked");
                    unsafe {
                        new_token.switch();
                        libc::longjmp(&new_jmp_buf, 1)
                    }
                }
            };

            match setjmp() {
                Some(old_jmp_buf) => switch(old_jmp_buf),
                None => { mem::forget(switch); true }
            }
        },

        None => false
    }
}

impl BlockedThread {
    pub fn resume(self) {
        let mut state = lock_sched!();
        state.threads.push_back(self);
    }
}

pub fn schedule() {
    block(move |thread| thread.resume());
}

pub fn exit(code: i32) -> ! {
    let exited = lock_sched!().current.exited.clone();
    exited.resolve(code);

    block(move |thread| {
        let mut state = lock_sched!();
        state.garbage_stacks.push(thread.1.stack);
    });

    panic!("exit: no more threads")
}

fn spawn_inner<'a>(process: Arc<Process>, start: Box<FnBox() + 'a>) -> Deferred<i32> {
    let stack_len = 4096;
    let stack_base_ptr = unsafe { heap::allocate(stack_len, 16) };
    let b: Box<FnBox() + 'a> = Box::new(start);
    let jmp_buf = thread::new_jmp_buf(b, unsafe { stack_base_ptr.offset(stack_len as isize) });
    let thread = Thread::new(process, unsafe { slice::from_raw_parts_mut(stack_base_ptr, stack_len) });
    let exited = thread.exited.clone();
    BlockedThread(jmp_buf, thread).resume();
    exited
}

pub fn spawn_remote<T>(process: Arc<Process>, start: T) -> Deferred<i32> where T : FnOnce() -> i32 {
    let start = || { exit(start()) };
    spawn_inner(process, Box::new(start))
}

pub fn spawn<T>(start: T) -> Deferred<i32> where T : FnOnce() -> i32 {
    let process = lock_sched!().current.process.clone();
    spawn_remote(process, start)
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
