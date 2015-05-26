use core::ops::FnOnce;
use libc::{self,jmp_buf};
use spin::RwLock;
use std::boxed::{Box,FnBox};
use std::collections::LinkedList;
use std::clone::Clone;
use std::marker::Copy;
use std::mem;
use std::option::Option::{self,Some,None};
use std::sync::Arc;
use std::vec::{self,Vec};
use super::arch::thread;

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

pub struct Thread {
    stack: vec::Vec<isize>
}

struct DeferredState<A> {
    result: Option<A>,
    waiters: LinkedList<(jmp_buf, Thread)>
}

struct SchedulerState {
    current: Thread,
    threads: LinkedList<(jmp_buf, bool, Thread)>
}

pub struct Deferred<'a, A> {
    scheduler: &'a Scheduler,
    state: Arc<RwLock<DeferredState<A>>>
}

pub struct Scheduler {
    state: RwLock<SchedulerState>
}

impl Scheduler {
    pub fn new() -> Scheduler {
        let state = SchedulerState {
            current: Thread { stack: Vec::new() },
            threads: LinkedList::new()
        };
        Scheduler { state: RwLock::new(state) }
    }

    pub fn schedule(&self) {
        let mut state = self.state.write();
        match state.threads.pop_front() {
            Some((new_jmp_buf, new_thread_holds_lock, new_current)) => {
                log!("schedule(longjmp to rip = {:x}, rsp = {:x})", new_jmp_buf.rip, new_jmp_buf.rsp);
                let old_current = mem::replace(&mut state.current, new_current);

                match setjmp() {
                    Some(old_jmp_buf) => {
                        state.threads.push_back((old_jmp_buf, true, old_current));

                        if !new_thread_holds_lock {
                            mem::drop(state);
                        }

                        unsafe {
                            libc::longjmp(&new_jmp_buf, 1);
                        }
                    },

                    None => { }
                }
            },

            None => { }
        };
    }

    pub fn exit_current(&self) -> ! {
        let mut state = self.state.write();
        let (new_jmp_buf, new_thread_holds_lock, new_current) = state.threads.pop_front().expect("no more threads");
        log!("exit_current(longjmp to rip = {:x}, rsp = {:x})", new_jmp_buf.rip, new_jmp_buf.rsp);
        state.current = new_current;
        if !new_thread_holds_lock {
            mem::drop(state);
        }

        unsafe {
            libc::longjmp(&new_jmp_buf, 1);
        }
    }

    fn get_deferred<A>(&self, dstate: &RwLock<DeferredState<A>>) -> A where A : Copy {
        loop {
            let mut dstate = dstate.write();
            match dstate.result {
                Some(result) => { return result; },
                None => { }
            }

            let mut state = self.state.write();
            let (new_jmp_buf, new_thread_holds_lock, new_current) = state.threads.pop_front().expect("no more threads");
            log!("block(longjmp to rip = {:x}, rsp = {:x})", new_jmp_buf.rip, new_jmp_buf.rsp);

            let old_current = mem::replace(&mut state.current, new_current);

            match setjmp() {
                Some(old_jmp_buf) => {
                    dstate.waiters.push_back((old_jmp_buf, old_current));

                    if !new_thread_holds_lock {
                        mem::drop(state);
                    }

                    mem::drop(dstate);

                    unsafe {
                        libc::longjmp(&new_jmp_buf, 1);
                    }
                },

                None => { }
            }
        }
    }

    fn resolve_deferred<A>(&self, dstate: &RwLock<DeferredState<A>>, result: A) where A : Copy {
        let mut dstate = dstate.write();
        let mut state = self.state.write();
        dstate.result = Some(result);

        let waiters = mem::replace(&mut dstate.waiters, LinkedList::new());
        for (jmp_buf, thread) in waiters {
            state.threads.push_back((jmp_buf, true, thread));
        }
    }

    fn spawn_inner<'a>(&'a self, start: Box<FnBox() + 'a>) {
        let mut stack = vec::from_elem(0, 4096);
        let jmp_buf = thread::new_jmp_buf(Box::new(start), &mut stack);
        log!("spawn(rip = {:x}, rsp = {:x})", jmp_buf.rip, jmp_buf.rsp);

        let thread = Thread { stack: stack };
        {
            let mut state = self.state.write();
            state.threads.push_back((jmp_buf, false, thread));
        }
    }

    pub fn spawn<T, A>(&self, start: T) -> Deferred<A> where T : FnOnce() -> A, A : Copy {
        let dstate = Arc::new(RwLock::new(DeferredState {
            result: None,
            waiters: LinkedList::new()
        }));

        let start = || {
            let result = start();
            self.resolve_deferred(&dstate, result);
            self.exit_current();
        };

        self.spawn_inner(Box::new(start));

        Deferred { scheduler: &self, state: dstate.clone() }
    }
}

pub trait Promise<A> {
    fn get(&self) -> A;
    // fn then<B>(f: FnOnce(A) -> B) -> Promise<B>;
}

impl<'a, A> Deferred<'a, A> where A : Copy {
    pub fn resolve(&self, result: A) {
        self.scheduler.resolve_deferred(&self.state, result)
    }
}

impl<'a, A> Promise<A> for Deferred<'a, A> where A : Copy {
    fn get(&self) -> A {
        self.scheduler.get_deferred(&self.state)
    }
}

test! {
    fn can_spawn_exit_thread() {
        let thread_fn = || { 123 };
        let scheduler = Scheduler::new();
        let d = scheduler.spawn(thread_fn);
        assert_eq!(123, d.get());
    }

    fn can_spawn_exit_two_threads() {
        let thread1_fn = || { 123 };
        let thread2_fn = || { 456 };
        let scheduler = Scheduler::new();
        let d1 = scheduler.spawn(thread1_fn);
        let d2 = scheduler.spawn(thread2_fn);
        assert_eq!(123, d1.get());
        assert_eq!(456, d2.get());
    }
}
