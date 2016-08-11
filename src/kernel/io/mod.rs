//! Input and output.

pub mod flat_map;
pub mod pipe;

mod nodes;

use core::result;
use deferred::Deferred;
use io::nodes::PromiseNode;
use prelude::*;
use syscall::Result;

pub use self::flat_map::FlatMap;
pub use self::pipe::Pipe;

/// Allows for reading bytes from a source.
pub trait Read {
    fn read(&self, buf: &mut [u8]) -> Result<usize>;
}

/// A trait for objects which are byte-oriented sinks.
pub trait Write {
    fn write(&self, buf: &[u8]) -> Result<usize>;
}

/// A computation that might eventually resolve to a value of type `T`.
pub struct Promise<T>(Box<PromiseNode<T>>);

impl<T: 'static> Promise<T> {
    /// Creates a promise from a kernel `Deferred` object. The promise is resolved once the deferred is resolved.
    pub fn new(d: Deferred<T>) -> Self {
        Promise(Box::new(nodes::deferred(d)))
    }

    /// Creates a promise from a constant value. The promise is resolved immediately.
    pub fn resolved(value: T) -> Self {
        Promise(Box::new(nodes::resolved(value)))
    }

    /// Blocks until the promise is resolved, then returns the value within.
    pub fn get(self) -> T {
        self.0.get()
    }

    /// Attempts to get the value inside the promise.
    ///
    /// If the promise is resolved, consumes the promise and returns `Some(value)`. Otherwise,
    /// returns `Err` containing the original promise.
    pub fn try_get(self) -> result::Result<T, Self> {
        self.0.try_get().map_err(|node| Promise(node))
    }

    /// Returns a new promise that applies a function to the value inside the promise.
    pub fn then<U, F: FnOnce(T) -> U + 'static>(self, f: F) -> Promise<U> {
        Promise(Box::new(self.0.map(f)))
    }
}

impl<T: 'static> Promise<Promise<T>> {
    /// Given a promise that resolves to `Promise<T>`, returns a promise that resolves to `T`.
    pub fn unwrap(self) -> Promise<T> {
        Promise(Box::new(self.0.unwrap()))
    }
}

/// Allows for reading bytes asynchronously from a source.
pub trait AsyncRead {
    fn read_async(&self, buf: Vec<u8>) -> Promise<Result<Vec<u8>>>;
}

impl<T: AsyncRead> Read for T {
    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let p = self.read_async(vec![0; buf.len()]);
        let v = try!(p.get());
        buf.copy_from_slice(&v);
        Ok(v.len())
    }
}
