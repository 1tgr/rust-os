use async::Promise;
use collections::vec_deque::VecDeque;
use core::cmp;
use mutex::Mutex;
use prelude::*;
use thread::Deferred;

struct IoRequest {
    buf: Vec<u8>,
    d: Deferred<Result<(Vec<u8>, usize), &'static str>>,
    current: usize
}

impl IoRequest {
    pub fn new(buf: Vec<u8>, d: Deferred<Result<(Vec<u8>, usize), &'static str>>) -> Self {
        IoRequest {
            buf: buf,
            d: d,
            current: 0
        }
    }

    pub fn fulfil(mut self, data: &mut VecDeque<u8>) -> Option<Self> {
        {
            let len = cmp::min(self.buf.len(), data.len());
            let right = data.split_off(len);

            for (i, b) in data.iter().enumerate() {
                self.buf[self.current + i] = *b
            }

            self.current += len;
            *data = right
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
    requests: Mutex<VecDeque<IoRequest>>
}

impl ByteDevice {
    pub fn new() -> Self {
        ByteDevice {
            requests: Mutex::new(VecDeque::new())
        }
    }

    pub fn fulfil(&self, data: &mut VecDeque<u8>) {
        loop {
            let mut requests = lock!(self.requests);
            let request: IoRequest =
                match requests.pop_front() {
                    Some(request) => request,
                    None => { return; }
                };

            if let Some(request) = request.fulfil(data) {
                requests.push_front(request);
            }

            if data.len() == 0 {
                return;
            }
        }
    }

    pub fn queue(&self, buf: Vec<u8>, d: Deferred<Result<(Vec<u8>, usize), &'static str>>) {
        lock!(self.requests).push_back(IoRequest::new(buf, d))
    }

    pub fn read_async(&self, queue: &mut VecDeque<u8>, buf: Vec<u8>) -> Box<Promise<Result<(Vec<u8>, usize), &'static str>>> {
        let d = Deferred::new();
        self.queue(buf, d.clone());
        self.fulfil(queue);
        Box::new(d)
    }
}

fn test_read(queue: &mut VecDeque<u8>, device: &ByteDevice, expected: &[u8]) {
    let buf = vec![0; expected.len()];
    let d = device.read_async(queue, buf);
    let (buf, len) = d.try_get().unwrap().unwrap();
    assert_eq!(expected.len(), len);

    let mut v = Vec::<u8>::new();
    v.extend(expected);
    assert_eq!(v, buf);
}

test! {
    fn can_read_nothing() {
        let mut queue = VecDeque::new();
        let device = ByteDevice::new();
        test_read(&mut queue, &device, b"");
    }

    fn can_read_chunks() {
        let mut queue = VecDeque::new();
        queue.extend(b"hello");

        let device = ByteDevice::new();
        test_read(&mut queue, &device, b"h");
        test_read(&mut queue, &device, b"ell");
        test_read(&mut queue, &device, b"o");
        assert_eq!(0, queue.len());
    }

    fn can_read_everything() {
        let mut queue = VecDeque::new();
        queue.extend(b"hello");

        let device = ByteDevice::new();
        test_read(&mut queue, &device, b"hello");
        assert_eq!(0, queue.len());
    }

    fn blocks_when_out_of_data() {
        let mut queue = VecDeque::new();
        queue.extend(b"hello");

        let device = ByteDevice::new();
        let d = device.read_async(&mut queue, vec![0; 10]);
        assert_eq!(None, d.try_get());
    }
}
