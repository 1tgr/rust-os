use prelude::*;

pub trait Promise<A> {
    fn get(&self) -> A;
    fn try_get(&self) -> Option<A>;
    // fn then<B>(f: FnOnce(A) -> B) -> Promise<B>;
}

pub trait AsyncRead {
    fn read_async(&self, buf: Vec<u8>) -> Box<Promise<Result<(Vec<u8>, usize), &'static str>>>;
}
