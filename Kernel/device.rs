use ::thread::{Deferred,Scheduler};
use spin::Mutex;
use std::cmp;
use std::collections::VecDeque;
use std::slice::bytes;
use std::sync::Arc;

pub trait Device {
    fn read_async(&self, buf: Vec<u8>) -> Deferred<Result<(Vec<u8>, usize), &'static str>>;
}

struct IoRequest {
    buf: Vec<u8>,
    d: Deferred<Result<(Vec<u8>, usize), &'static str>>,
    current: usize
}

impl IoRequest {
    pub fn new(buf: Vec<u8>, d: Deferred<Result<(Vec<u8>, usize), &'static str>>) -> IoRequest {
        IoRequest {
            buf: buf,
            d: d,
            current: 0
        }
    }

    pub fn fulfil(mut self, data: &mut &[u8]) -> Option<IoRequest> {
        {
            let len = cmp::min(self.buf.len(), data.len());
            let (data1, data2) = data.split_at(len);
            *data = data2;

            let buf1 = &mut self.buf[self.current .. self.current + len];
            bytes::copy_memory(data1, buf1);
            self.current += len;
        }

        if self.current >= self.buf.len() {
            self.d.resolve(Ok((self.buf, self.current)));
            None
        } else {
            Some(self)
        }
    }
}

pub struct ByteDevice {
    scheduler: Arc<Scheduler>,
    requests: Mutex<VecDeque<IoRequest>>
}

impl ByteDevice {
    pub fn new(scheduler: Arc<Scheduler>) -> ByteDevice {
        ByteDevice {
            scheduler: scheduler,
            requests: Mutex::new(VecDeque::new())
        }
    }

    pub fn fulfil(&self, data: &mut &[u8]) {
        loop {
            if data.len() == 0 {
                return;
            }

            let mut requests = lock!(self.requests);
            let request: IoRequest =
                match requests.pop_front() {
                    Some(request) => request,
                    None => { return; }
                };

            if let Some(request) = request.fulfil(data) {
                requests.push_front(request);
            }
        }
    }
}

impl Device for ByteDevice {
    fn read_async(&self, buf: Vec<u8>) -> Deferred<Result<(Vec<u8>, usize), &'static str>> {
        let d = Deferred::new(self.scheduler.clone());
        lock!(self.requests).push_back(IoRequest::new(buf, d.clone()));
        d
    }
}
