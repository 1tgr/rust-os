use alloc::arc::Arc;
use collections::vec_deque::VecDeque;
use core::mem;
use spin::Mutex;
use kobj::KObj;
use thread::{self,BlockedThread};

struct DeferredState<A> {
    result: Option<A>,
    waiters: VecDeque<BlockedThread>
}

pub struct Deferred<A> {
    state: Arc<Mutex<DeferredState<A>>>
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
        while let Some(thread) = waiters.pop_front() {
            thread.resume();
        }
    }

    pub fn get(self) -> A {
        loop {
            let mut dstate = lock!(self.state);
            match mem::replace(&mut dstate.result, None) {
                Some(result) => { return result; },
                None => (),
            }

            let found_new_thread = thread::block(move |thread| {
                dstate.waiters.push_back(thread);
            });

            assert!(found_new_thread, "block: no more threads");
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

impl<A> Clone for Deferred<A> {
    fn clone(&self) -> Self {
        Deferred { state: self.state.clone() }
    }
}

impl KObj for Deferred<i32> {
    fn deferred_i32(&self) -> Option<Self> {
        Some(self.clone())
    }
}
