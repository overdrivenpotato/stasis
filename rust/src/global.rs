//! Type-level safe mutable global access.
//!
//! This is useful for asynchronous functions and memoizing modules.

use std::{
    sync::{Arc, Mutex, MutexGuard, Once, ONCE_INIT},
    ops::{Deref, DerefMut},
    cell::UnsafeCell,
    mem::ManuallyDrop,
};

/// A global value.
///
/// Handles to this value can be obtained with the [`lock`] method.
pub struct Global<T> {
    once: Once,
    inner: UnsafeCell<Option<Arc<Mutex<T>>>>,
}

unsafe impl<T> Sync for Global<T> {}

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

impl<T: Default> Global<T> {
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
    pub fn lock(&'static self) -> GlobalLock<T> where T: 'static {
        self.ensure_exists();

        let ptr = self.inner.get();

        // This is safe as we're
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
        unsafe {
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
