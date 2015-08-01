use ::thread::{Deferred,Scheduler};
use spin::Mutex;
use std::cmp;
use std::collections::VecDeque;
use std::slice::bytes;

pub trait Device<'a> {
    fn read_async(&self, buf: Vec<u8>) -> Deferred<'a, Result<(Vec<u8>, usize), &'static str>>;
}

struct IoRequest<'a> {
    buf: Vec<u8>,
    d: Deferred<'a, Result<(Vec<u8>, usize), &'static str>>,
    current: usize
}

impl<'a> IoRequest<'a> {
    pub fn new(buf: Vec<u8>, d: Deferred<'a, Result<(Vec<u8>, usize), &'static str>>) -> Self {
        IoRequest {
            buf: buf,
            d: d,
            current: 0
        }
    }

    pub fn fulfil(mut self, data: &mut &[u8]) -> Option<IoRequest<'a>> {
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

pub struct ByteDevice<'a> {
    scheduler: &'a Scheduler,
    requests: Mutex<VecDeque<IoRequest<'a>>>
}

impl<'a> ByteDevice<'a> {
    pub fn new(scheduler: &'a Scheduler) -> Self {
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
            let request: IoRequest<'a> =
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

impl<'a> Device<'a> for ByteDevice<'a> {
    fn read_async(&self, buf: Vec<u8>) -> Deferred<'a, Result<(Vec<u8>, usize), &'static str>> {
        let d = Deferred::new(self.scheduler.clone());
        lock!(self.requests).push_back(IoRequest::new(buf, d.clone()));
        d
    }
}
