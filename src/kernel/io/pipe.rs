use alloc::arc::Arc;
use collections::vec_deque::VecDeque;
use core::cmp;
use io::{AsyncRead,Promise};
use mutex::Mutex;
use prelude::*;
use syscall::Result;

pub struct Pipe<T, U> {
    input: Arc<T>,
    queue: Arc<Mutex<VecDeque<u8>>>,
    buf_len: usize,
    f: Arc<Fn(Vec<u8>) -> U>
}

struct State<T, U> {
    pipe: Pipe<T, U>,
    buf: Vec<u8>,
    current: usize
}

impl<T: AsyncRead + 'static, U: IntoIterator<Item=u8> + 'static> State<T, U> {
    fn read_async(mut self) -> Promise<Result<Vec<u8>>> {
        {
            let mut queue = lock!(self.pipe.queue);
            let end = cmp::min(self.buf.len(), self.current + queue.len());
            while self.current < end {
                let b = queue.pop_front().unwrap();
                self.buf[self.current] = b;
                self.current += 1;
            }
        }

        if self.current  == self.buf.len() {
            return Promise::resolved(Ok(self.buf));
        }

        self.pipe.input
            .read_async(vec![0; self.pipe.buf_len])
            .then(move |result| {
                if let Ok(data) = result {
                    let output = (self.pipe.f)(data);

                    {
                        let mut queue = lock!(self.pipe.queue);
                        for b in output {
                            queue.push_back(b);
                        }
                    }

                    self.read_async()
                }
                else {
                    Promise::resolved(result)
                }
            })
        .unwrap()
    }
}

impl<T, U> Pipe<T, U> {
    pub fn new<F: Fn(Vec<u8>) -> U + 'static>(input: Arc<T>, buf_len: usize, f: F) -> Self {
        Pipe {
            input: input,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            buf_len: buf_len,
            f: Arc::new(f)
        }
    }
}

impl<T: AsyncRead + 'static, U: IntoIterator<Item=u8> + 'static> AsyncRead for Pipe<T, U> {
    fn read_async(&self, buf: Vec<u8>) -> Promise<Result<Vec<u8>>> {
        let state = State {
            pipe: Pipe {
                input: self.input.clone(),
                queue: self.queue.clone(),
                buf_len: self.buf_len,
                f: self.f.clone()
            },
            buf: buf,
            current: 0
        };

        state.read_async()
    }
}
