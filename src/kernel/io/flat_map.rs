use crate::io::{AsyncRead, Promise};
use crate::kobj::KObjRef;
use crate::prelude::*;
use crate::spin::Mutex;
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use syscall::Result;

struct FlatMapShared {
    queue: Mutex<VecDeque<u8>>,
    buf_len: usize,
    collect: Box<dyn Fn(Vec<u8>) -> Vec<u8>>,
    finished: Box<dyn Fn(&mut Vec<u8>) -> Option<Vec<u8>>>,
}

/// An adaptor which will modify the data produced by a reader.
pub struct FlatMap {
    shared: Arc<FlatMapShared>,
    input: KObjRef<dyn AsyncRead>,
}

fn read_async_inner(
    shared: Arc<FlatMapShared>,
    input: KObjRef<dyn AsyncRead>,
    mut buf: Vec<u8>,
    len: usize,
) -> Promise<Result<Vec<u8>>> {
    {
        let mut queue = lock!(shared.queue);
        while buf.len() < len {
            if let Some(b) = queue.pop_front() {
                buf.push(b);
            } else {
                break;
            }
        }

        if let Some(remainder) = (shared.finished)(&mut buf) {
            queue.extend(remainder);
            return Promise::resolved(Ok(buf));
        } else if buf.len() > 0 || len == 0 {
            return Promise::resolved(Ok(buf));
        }
    }

    input
        .read_async(vec![0; shared.buf_len])
        .then(move |result| {
            if let Ok(data) = result {
                let output = (shared.collect)(data);
                lock!(shared.queue).extend(output);
                read_async_inner(shared, input, buf, len)
            } else {
                Promise::resolved(result)
            }
        })
        .unwrap()
}

impl FlatMap {
    pub fn new<F: Fn(Vec<u8>) -> Vec<u8> + 'static, G: Fn(&mut Vec<u8>) -> Option<Vec<u8>> + 'static>(
        input: KObjRef<dyn AsyncRead>,
        buf_len: usize,
        collect: F,
        finished: G,
    ) -> Self {
        FlatMap {
            shared: Arc::new(FlatMapShared {
                queue: Mutex::new(VecDeque::new()),
                buf_len,
                collect: Box::new(collect),
                finished: Box::new(finished),
            }),
            input,
        }
    }
}

impl AsyncRead for FlatMap {
    fn read_async(&self, mut buf: Vec<u8>) -> Promise<Result<Vec<u8>>> {
        let len = buf.len();
        buf.truncate(0);
        read_async_inner(self.shared.clone(), self.input.clone(), buf, len)
    }
}
