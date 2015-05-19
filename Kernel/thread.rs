use core::ops::FnOnce;
use libc::{self,jmp_buf};
use spin::RwLock;
use std::boxed::{Box,FnBox};
use std::collections::LinkedList;
use std::mem;
use std::option::Option::{Some,None};
use std::vec::{self,Vec};
use super::arch::thread;

pub struct Thread {
    stack: vec::Vec<isize>
}

struct SchedulerState {
    current: Thread,
    threads: LinkedList<(jmp_buf, bool, Thread)>
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
                let mut old_jmp_buf = unsafe { mem::uninitialized() };

                unsafe {
                    if libc::setjmp(&mut old_jmp_buf) == 0 {
                        state.threads.push_back((old_jmp_buf, true, old_current));

                        if !new_thread_holds_lock {
                            mem::drop(state);
                        }

                        libc::longjmp(&new_jmp_buf, 1);
                    }
                }
            },

            None => { }
        };
    }

    pub fn exit_current(&self) -> ! {
        let mut state = self.state.write();
        match state.threads.pop_front() {
            Some((new_jmp_buf, new_thread_holds_lock, new_current)) => {
                log!("exit_current(longjmp to rip = {:x}, rsp = {:x})", new_jmp_buf.rip, new_jmp_buf.rsp);
                state.current = new_current;
                if !new_thread_holds_lock {
                    mem::drop(state);
                }

                unsafe {
                    libc::longjmp(&new_jmp_buf, 1);
                }
            },

            None => panic!("no more threads")
        };
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

    pub fn spawn<T>(&self, start: T) where T : FnOnce() -> () {
        let start = || {
            start();
            self.exit_current();
        };

        self.spawn_inner(Box::new(start))
    }
}
