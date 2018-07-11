//! Type-level safe mutable global access.
//!
//! This is useful for asynchronous functions and memoizing modules.

use std::{
    sync::{Arc, Mutex, MutexGuard, Once, ONCE_INIT},
    ops::{Deref, DerefMut},
    cell::UnsafeCell,
    mem::ManuallyDrop,
};

/// A global value wrapped in a [`Mutex`].
///
/// Handles to this value can be obtained with the [`Global::lock`] method.
///
/// [`Mutex`]: std::sync::Mutex
pub struct Global<T> {
    once: Once,
    inner: UnsafeCell<Option<Arc<Mutex<T>>>>,
}

// The inner value is only used to make an immutable call to `.clone()`. The
// only time it is mutated is within the `Once` guard. This means all threads
// will attempt to get *immutable* access and block until only one thread as
// succeeded. That makes this `impl` safe only if `.ensure_exists()` is called
// whenever accessing the inner `UnsafeCell` value.
//
// This bound is on `T: Send` as `Mutex<T>` requires it to implement `T: Sync`.
// Because the mutex is in a static position it must be sync, so we need to
// ensure this bound is satisfied.
unsafe impl<T> Sync for Global<T> where T: Send {}

impl<T: Default> Global<T> {
    /// Ensure the inner value exists.
    ///
    /// This method *must* be called when accessing the inner `UnsafeCell`.
    fn ensure_exists(&'static self) {
        self.once.call_once(|| {
            let ptr = self.inner.get();

            // This is safe as this assignment can only be called once, hence no
            // hint of race conditions. Other threads will be blocked until this
            // is done.
            unsafe {
                if (*ptr).is_none() {
                    *ptr = Some(Arc::new(Mutex::new(T::default())));
                }
            }
        });
    }
}

impl<T: Default + Send + 'static> Global<T> {
    /// The initial global value.
    pub const INIT: Global<T> = Global {
        once: ONCE_INIT,
        inner: UnsafeCell::new(None),
    };

    /// Run a closure on the inner value.
    ///
    /// This will return the closure's return type. This is a cheap function
    /// call.
    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut *self.lock())
    }

    /// Obtain a lock on the inner reference.
    ///
    /// Because WebAssembly is currently single threaded, this operation is
    /// cheap. This may change in the future, however this code will continue to
    /// work on multi-threaded systems.
    ///
    /// This method will block the current thread until the lock is available.
    /// If this is called recursively in WebAssembly, it will panic.
    pub fn lock(&'static self) -> GlobalLock<T> {
        // Important: this *must* be called before accessing the inner pointer.
        self.ensure_exists();

        let ptr = self.inner.get() as *const Option<_>;

        // This is safe as we already called `ensure_exists`.
        let opt = unsafe { (*ptr).clone() };

        GlobalLock::new(opt.unwrap())
    }
}

/// A handle to some global value of type `T`.
pub struct GlobalLock<T: 'static> {
    // These are marked manually drop to specify drop order. In a perfect world,
    // the guard would bear the lifetime of the mutex, however that requires
    // rust to have self-referential structs, which it currently does not have.
    mutex: ManuallyDrop<Arc<Mutex<T>>>,
    guard: ManuallyDrop<MutexGuard<'static, T>>,
}

impl<T: 'static> Drop for GlobalLock<T> {
    fn drop(&mut self) {
        // Drop the guard *before* the mutex.
        unsafe {
            ManuallyDrop::drop(&mut self.guard);
            ManuallyDrop::drop(&mut self.mutex);
        }
    }
}

impl<T: 'static> GlobalLock<T> {
    /// Construct a new `GlobalLock` with a reference-counted mutex.
    fn new(mut mutex: Arc<Mutex<T>>) -> Self {
        // Both the guard and the mutex are moved into the lock. Rust does not
        // support self-referential lifetimes so we must use unsafe code here.
        unsafe {
            // Remove the lifetime constraints on a borrow.
            let ptr = &mut mutex as *mut Arc<Mutex<T>>;

            // This should never fail.
            let guard = (*ptr)
                .lock()
                .unwrap();

            GlobalLock {
                guard: ManuallyDrop::new(guard),
                mutex: ManuallyDrop::new(mutex),
            }
        }
    }
}

impl<T: 'static> Deref for GlobalLock<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.guard
    }
}

impl<T: 'static> DerefMut for GlobalLock<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.guard
    }
}

#[cfg(test)]
mod test {
    use std::{
        thread,
        sync::mpsc,
        time::Duration,
    };

    use super::Global;

    #[test]
    fn no_race_condition() {
        static NUM: Global<i32> = Global::INIT;

        let mut v = Vec::new();

        for _ in 0..1000 {
            v.push(thread::spawn(|| {
                for _ in 0..100 {
                    *NUM.lock() += 1;
                }
            }));
        }

        for thread in v {
            thread.join().unwrap();
        }

        assert_eq!(*NUM.lock(), 100_000);
    }

    // Ensure a lock will block.
    #[test]
    fn no_race_extended_lock() {
        static NUM: Global<i32> = Global::INIT;

        let (tx, rx) = mpsc::channel();

        let t1 = thread::spawn(move || {
            let mut lock = NUM.lock();

            // Go.
            tx.send(()).unwrap();

            thread::sleep(Duration::new(0, 1000000));

            *lock += 1;
        });

        let t2 = thread::spawn(move || {
            // Wait for the signal.
            let () = rx.recv().unwrap();

            let mut lock = NUM.lock();

            *lock += 1;
        });


        t1.join().unwrap();
        t2.join().unwrap();

        assert_eq!(*NUM.lock(), 2);
    }
}
