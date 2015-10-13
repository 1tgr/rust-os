use alloc::arc::Arc;
use collections::vec_deque::VecDeque;
use core::cmp;
use io::{AsyncRead,Promise};
use mutex::Mutex;
use prelude::*;
use syscall::Result;

/// An adaptor which will modify the data produced by a reader.
pub struct Pipe<T, U> {
    input: Arc<T>,
    queue: Arc<Mutex<VecDeque<u8>>>,
    buf_len: usize,
    collect: Arc<Fn(Vec<u8>) -> U>,
    finished: Arc<Fn(&mut Vec<u8>) -> Option<Vec<u8>>>
}

impl<T, U> Pipe<T, U> {
    pub fn new<F: Fn(Vec<u8>) -> U + 'static, G: Fn(&mut Vec<u8>) -> Option<Vec<u8>> + 'static>(input: Arc<T>, buf_len: usize, collect: F, finished: G) -> Self {
        Pipe {
            input: input,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            buf_len: buf_len,
            collect: Arc::new(collect),
            finished: Arc::new(finished)
        }
    }
}

impl<T: AsyncRead + 'static, U: IntoIterator<Item=u8> + 'static> Pipe<T, U> {
    fn read_async_inner(self, mut buf: Vec<u8>, mut current: usize) -> Promise<Result<Vec<u8>>> {
        {
            let mut queue = lock!(self.queue);
            let end = cmp::min(buf.len(), current + queue.len());
            while current < end {
                let b = queue.pop_front().unwrap();
                buf[current] = b;
                current += 1;
            }

            if let Some(remainder) = (self.finished)(&mut buf) {
                queue.extend(remainder);
                return Promise::resolved(Ok(buf));
            }
            else if current  == buf.len() {
                return Promise::resolved(Ok(buf));
            }
        }

        self.input
            .read_async(vec![0; self.buf_len])
            .then(move |result| {
                if let Ok(data) = result {
                    let output = (self.collect)(data);
                    lock!(self.queue).extend(output);
                    self.read_async_inner(buf, current)
                }
                else {
                    Promise::resolved(result)
                }
            })
        .unwrap()
    }
}

impl<T: AsyncRead + 'static, U: IntoIterator<Item=u8> + 'static> AsyncRead for Pipe<T, U> {
    fn read_async(&self, buf: Vec<u8>) -> Promise<Result<Vec<u8>>> {
        let pipe = Pipe {
            input: self.input.clone(),
            queue: self.queue.clone(),
            buf_len: self.buf_len,
            collect: self.collect.clone(),
            finished: self.finished.clone()
        };

        pipe.read_async_inner(buf, 0)
    }
}
