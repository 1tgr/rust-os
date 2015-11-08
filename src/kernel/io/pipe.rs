use collections::vec_deque::VecDeque;
use core::cmp;
use deferred::Deferred;
use io::{AsyncRead,Promise,Read,Write};
use mutex::Mutex;
use prelude::*;
use process::KObj;
use syscall::Result;

struct IoRequest {
    buf: Vec<u8>,
    d: Deferred<Result<Vec<u8>>>,
    current: usize
}

impl IoRequest {
    pub fn new(buf: Vec<u8>, d: Deferred<Result<Vec<u8>>>) -> Self {
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
            self.d.resolve(Ok(self.buf));
            None
        } else {
            Some(self)
        }
    }
}

pub struct Pipe {
    data: Mutex<VecDeque<u8>>,
    requests: Mutex<VecDeque<IoRequest>>
}

impl Pipe {
    pub fn new() -> Self {
        Pipe {
            data: Mutex::new(VecDeque::new()),
            requests: Mutex::new(VecDeque::new())
        }
    }

    pub fn queue_len(&self) -> usize {
        lock!(self.data).len()
    }

    fn fulfil(&self) -> bool {
        let mut data = lock!(self.data);
        loop {
            let mut requests = lock!(self.requests);
            let request: IoRequest =
                match requests.pop_front() {
                    Some(request) => request,
                    None => { return false; }
                };

            if let Some(request) = request.fulfil(&mut data) {
                requests.push_front(request);
            }

            if data.len() == 0 {
                return !requests.is_empty();
            }
        }
    }
}

impl AsyncRead for Pipe {
    fn read_async(&self, buf: Vec<u8>) -> Promise<Result<Vec<u8>>> {
        let d = Deferred::new();
        lock!(self.requests).push_back(IoRequest::new(buf, d.clone()));
        self.fulfil();
        Promise::new(d)
    }
}

impl Write for Pipe {
    fn write(&self, buf: &[u8]) -> Result<usize> {
        lock!(self.data).extend(buf);
        self.fulfil();
        Ok(buf.len())
    }
}

impl KObj for Pipe {
    fn async_read(&self) -> Option<&AsyncRead> {
        Some(self)
    }

    fn read(&self) -> Option<&Read> {
        Some(self)
    }

    fn write(&self) -> Option<&Write> {
        Some(self)
    }
}

#[cfg(feature = "test")]
pub mod test {
    use io::{AsyncRead,Write};
    use prelude::*;
    use super::*;

    fn test_read(pipe: &Pipe, expected: &[u8]) {
        let buf = vec![0; expected.len()];
        let d = pipe.read_async(buf);
        let buf = d.try_get().unwrap_or_else(|_| panic!("didn't expect to block")).unwrap();
        let mut v = Vec::<u8>::new();
        v.extend(expected);
        assert_eq!(v, buf);
    }

    test! {
        fn can_read_nothing() {
            let pipe = Pipe::new();
            test_read(&pipe, b"");
        }

        fn can_read_chunks() {
            let pipe = Pipe::new();
            pipe.write(b"hello").unwrap();
            test_read(&pipe, b"h");
            test_read(&pipe, b"ell");
            test_read(&pipe, b"o");
            assert_eq!(0, pipe.queue_len());
        }

        fn can_read_everything() {
            let pipe = Pipe::new();
            pipe.write(b"hello").unwrap();
            test_read(&pipe, b"hello");
            assert_eq!(0, pipe.queue_len());
        }

        fn blocks_when_out_of_data() {
            let pipe = Pipe::new();
            pipe.write(b"hello").unwrap();

            let d = pipe.read_async(vec![0; 10]);
            assert!(d.try_get().is_err());
        }

        fn can_write_twice() {
            let pipe = Pipe::new();
            pipe.write(b"hello ").unwrap();
            test_read(&pipe, b"hel");
            pipe.write(b"world").unwrap();
            test_read(&pipe, b"lo wo");
            test_read(&pipe, b"r");
            assert_eq!(2, pipe.queue_len());
        }
    }
}
