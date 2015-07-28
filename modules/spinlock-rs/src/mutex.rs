#[cfg(not(feature = "no_std"))]
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
#[cfg(not(feature = "no_std"))]
use std::cell::UnsafeCell;
#[cfg(not(feature = "no_std"))]
use std::marker::Sync;
#[cfg(not(feature = "no_std"))]
use std::ops::{Drop, Deref, DerefMut};

#[cfg(feature = "no_std")]
use core::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
#[cfg(feature = "no_std")]
use core::cell::UnsafeCell;
#[cfg(feature = "no_std")]
use core::marker::Sync;
#[cfg(feature = "no_std")]
use core::ops::{Drop, Deref, DerefMut};

/// This type provides MUTual EXclusion based on spinning.
///
/// # Description
///
/// This structure behaves a lot like a normal Mutex. There are some differences:
///
/// - It may be used outside the runtime.
///   - A normal mutex will fail when used without the runtime, this will just lock
///   - When the runtime is present, it will call the deschedule function when appropriate
/// - No lock poisoning. When a fail occurs when the lock is held, no guarantees are made
///
/// When calling rust functions from bare threads, such as C `pthread`s, this lock will be very
/// helpful. In other cases however, you are encouraged to use the locks from the standard
/// library.
///
/// # Simple example
///
/// ```
/// use spin;
/// let spin_mutex = spin::Mutex::new(0);
///
/// // Modify the data
/// {
///     let mut data = spin_mutex.lock();
///     *data = 2;
/// }
///
/// // Read the data
/// let answer =
/// {
///     let data = spin_mutex.lock();
///     *data
/// };
///
/// assert_eq!(answer, 2);
/// ```
///
/// # Thread-safety example
///
/// ```
/// use spin;
/// use std::sync::{Arc, Barrier};
///
/// let numthreads = 1000;
/// let spin_mutex = Arc::new(spin::Mutex::new(0));
///
/// // We use a barrier to ensure the readout happens after all writing
/// let barrier = Arc::new(Barrier::new(numthreads + 1));
///
/// for _ in (0..numthreads)
/// {
///     let my_barrier = barrier.clone();
///     let my_lock = spin_mutex.clone();
///     std::thread::spawn(move||
///     {
///         let mut guard = my_lock.lock();
///         *guard += 1;
///
///         // Release the lock to prevent a deadlock
///         drop(guard);
///         my_barrier.wait();
///     });
/// }
///
/// barrier.wait();
///
/// let answer = { *spin_mutex.lock() };
/// assert_eq!(answer, numthreads);
/// ```
pub struct Mutex<T>
{
    lock: AtomicUsize,
    data: UnsafeCell<T>,
}

/// A guard to which the protected data can be accessed
///
/// When the guard falls out of scope it will release the lock.
pub struct MutexGuard<'a, T:'a>
{
    lock: &'a AtomicUsize,
    token: usize,
    data: &'a mut T,
}

unsafe impl<T> Sync for Mutex<T> {}

/// A Mutex which may be used statically.
///
/// ```
/// use spin::{self, STATIC_MUTEX_INIT};
///
/// static SPLCK: spin::StaticMutex = STATIC_MUTEX_INIT;
///
/// fn demo() {
///     let lock = SPLCK.lock();
///     // do something with lock
///     drop(lock);
/// }
/// ```
#[cfg(feature = "no_std")]
pub type StaticMutex = Mutex<()>;

/// A initializer for StaticMutex, containing no data.
#[cfg(feature = "no_std")]
pub const STATIC_MUTEX_INIT: StaticMutex = Mutex {
    lock: ATOMIC_USIZE_INIT,
    data: UnsafeCell::new(()),
};

impl<T> Mutex<T>
{
    /// Creates a new spinlock wrapping the supplied data.
    pub fn new(user_data: T) -> Mutex<T>
    {
        Mutex
        {
            lock: ATOMIC_USIZE_INIT,
            data: UnsafeCell::new(user_data),
        }
    }

    fn obtain_lock(&self, file_line: *const (&'static str, u32)) -> usize
    {
        let token = super::interrupts::disable();
        let null = 0;
        while self.lock.compare_and_swap(null, file_line as usize, Ordering::SeqCst) != null
        {
            // Do nothing
        }
        token
    }

    /// Locks the spinlock and returns a guard.
    ///
    /// The returned value may be dereferenced for data access
    /// and the lock will be dropped when the guard falls out of scope.
    ///
    /// ```
    /// let mylock = spin::Mutex::new(0);
    /// {
    ///     let mut data = mylock.lock();
    ///     // The lock is now locked and the data can be accessed
    ///     *data += 1;
    ///     // The lock is implicitly dropped
    /// }
    ///
    /// ```
    pub fn lock(&self, file_line: &'static (&'static str, u32)) -> MutexGuard<T>
    {
        let token = self.obtain_lock(file_line);
        MutexGuard
        {
            lock: &self.lock,
            token: token,
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Deallocates resources associated with this static mutex.
    ///
    /// This method is unsafe because it provides no guarantees that there are
    /// no active users of this mutex, and safety is not guaranteed if there are
    /// active users of this mutex.
    ///
    /// This method is required to ensure that there are no memory leaks on
    /// *all* platforms. It may be the case that some platforms do not leak
    /// memory if this method is not called, but this is not guaranteed to be
    /// true on all platforms.
    pub unsafe fn destroy(&'static self) {
        // nothing to do
    }
}

impl<'a, T> Deref for MutexGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}

impl<'a, T> DerefMut for MutexGuard<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T { &mut *self.data }
}

impl<'a, T> Drop for MutexGuard<'a, T>
{
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self)
    {
        self.lock.store(0, Ordering::SeqCst);
        super::interrupts::restore(self.token);
    }
}

#[macro_export]
macro_rules! lock {
    ($mutex:expr) => ({
        static FILE_LINE: (&'static str, u32) = (file!(), line!());
        $mutex.lock(&FILE_LINE)
    })
}
