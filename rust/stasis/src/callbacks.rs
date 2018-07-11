//! Easy handling of JavaScript callbacks.
//!
//! JavaScript callbacks are effectively global FFI and so handling these
//! callbacks involves a global callback manager. `Callbacks` performs this
//! task.
//!
//! `Callbacks` acts as a stack upon which you can push and pop items. Popping
//! an item requires you to pass an updated callback.
//!
//! ```rust,no_run
//! #[macro_use] extern crate stasis;
//!
//! use stasis::{
//!     console,
//!     Module,
//!     callbacks::{Callbacks, CallbackId},
//! };
//!
//! static CALLBACKS: Callbacks<()> = Callbacks::INIT;
//!
//! stasis! {{
//!     const DELAY: u32 = 1000;
//!
//!     // JavaScript setup.
//!     let m = Module::new();
//!     m.register_callback("done", |id: CallbackId| {
//!         CALLBACKS.push(id, ());
//!     });
//!     m.register("setTimeout", r#"
//!         function(id, ms) {
//!             var done = this.callbacks.done;
//!             setTimeout(function() { done(id) }, ms);
//!         }
//!     "#);
//!
//!     // Create and set up a callback listener.
//!     let id = CALLBACKS.create();
//!     CALLBACKS.listen(id, || console::log("Timeout finished"));
//!
//!     // This will print "Timeout finished" after 1000 milliseconds.
//!     let () = m.call("setTimeout", (id, DELAY));
//! }}
//! ```

use std::collections::{HashMap, VecDeque};
use std::cell::UnsafeCell;

use global::Global;
use once_nonstatic::Once;

/// A reference to a registered callback.
///
/// This callback may be waiting to be called, or it may have already been
/// called.
#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CallbackId(u32);

struct Inner<T> {
    current: u32,
    map: HashMap<CallbackId, Callback<T>>,
}

#[derive(Default)]
struct Callback<T> {
    notify: Option<Box<FnMut() + Send>>,
    stack: VecDeque<T>,
}

impl<T: Default> Inner<T> {
    fn pop(&mut self, id: CallbackId) -> Option<T> {
        self.map
            .get_mut(&id)?
            .stack
            .pop_front()
    }

    fn listen<F>(&mut self, id: CallbackId, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let cb = self.map
            .entry(id)
            .or_insert_with(Callback::default);

        let mut opt = Some(f);
        cb.notify = Some(Box::new(move || {
            let f = opt
                .take()
                .unwrap();

            f();
        }));
    }
}

impl<T> Default for Inner<T> {
    fn default() -> Self {
        Self {
            current: 0,
            map: HashMap::new(),
        }
    }
}

/// A lazily-initialized ID.
///
/// Constructing a callback generally takes some sort of setup code. This is a
/// wrapper around `CallbackId` that will call your initializer in a declarative
/// way.
pub struct LazyId {
    id: CallbackId,
    initialized: bool,
}

impl LazyId {
    /// Get the inner ID, running the given initializer if it has not been run.
    pub fn get_or_init<F>(&mut self, f: F) -> CallbackId
    where
        F: FnOnce(CallbackId)
    {
        if self.initialized {
            self.id
        } else {
            f(self.id);
            self.initialized = true;
            self.id
        }
    }
}

/// A callback manager.
///
/// This type can be created as a global on stable Rust.
// Nesting a `Global` in here directly triggers an ICE. The workaround here is
// to use an `UnsafeCell` + `Once` initializer.
//
// TODO: Remove the unsafety once the ICE is resolved.
// https://github.com/rust-lang/rust/issues/50518
pub struct Callbacks<T> {
    once: Once,
    inner: UnsafeCell<Option<Global<Inner<T>>>>,
}

// The bound here is taken directly from the `unsafe impl` of `Sync` on
// `Global<T>`. With this in mind, the impl is safe as `Once` guards access to
// the inner cell.
unsafe impl<T: Send> Sync for Callbacks<T> {}

impl<T: Default + 'static + Send> Callbacks<T> {
    pub const INIT: Callbacks<T> = Callbacks {
        once: Once::INIT,
        inner: UnsafeCell::new(None),
    };

    fn ensure_exists(&self) {
        // This is guarded by `Once`, which makes the access of the mutable
        // pointer safe.
        unsafe {
            let ptr = self.inner.get();

            self.once.call_once(move || {
                *ptr = Some(Global::new());
            });
        }
    }

    fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Inner<T>) -> R,
    {
        // Calling `ensure_exists` makes the unsafe code below safe.
        self.ensure_exists();

        unsafe {
            (*self.inner.get())
                .as_mut()
                .unwrap()
                .with(f)
        }
    }

    /// Create a unique `CallbackId`.
    pub fn create(&self) -> CallbackId {
        let id = self.with(|inner| {
            inner.current += 1;
            inner.current
        });

        CallbackId(id)
    }

    /// Create a lazily-initialized ID.
    pub fn lazy(&self) -> LazyId {
        LazyId {
            id: self.create(),
            initialized: false,
        }
    }

    /// Push a value onto the stack and notify the listener.
    pub fn push(&self, id: CallbackId, t: T) {
        let notify = self.with(|inner| {
            let cb = inner.map.get_mut(&id)?;

            cb.stack.push_back(t);
            cb.notify.take()
        });

        if let Some(mut f) = notify {
            f();
        }
    }

    /// Pop the next value off the stack.
    pub fn pop(&self, id: CallbackId) -> Option<T> {
        self.with(|inner| inner.pop(id))
    }

    /// Listen for push events.
    ///
    /// This will override the previous listener.
    pub fn listen<F>(&self, id: CallbackId, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.with(|inner| inner.listen(id, f))
    }

    /// Pop the next item off the stack and attach a listener for a future item.
    ///
    /// This is essentially a combination of `pop` and `listen`.
    pub fn pop_listen<F>(&self, id: CallbackId, f: F) -> Option<T>
    where
        F: FnOnce() + Send + 'static,
    {
        self.with(|inner| {
            inner.listen(id, f);
            inner.pop(id)
        })
    }
}
