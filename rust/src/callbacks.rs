//! Easy handling of JavaScript callbacks.
//!
//! This module is designed to work together with `global`. JavaScript
//! callbacks are effectively global FFI and so handling these callbacks involves
//! a global callback manager. `Callbacks` performs this task.
//!
//! ```rust,no_run
//! #[macro_use] extern crate stasis;
//!
//! use stasis::{
//!     console,
//!     Module,
//!     Global,
//!     callbacks::{Callbacks, CallbackId},
//! };
//!
//! static MODULE: Global<TestModule> = Global::INIT;
//! static CALLBACKS: Global<Callbacks<()>> = Global::INIT;
//!
//! struct TestModule(Module);
//!
//! impl Default for TestModule {
//!     fn default() -> Self {
//!         let m = Module::new();
//!
//!         m.register_callback("done", |id: CallbackId| {
//!             Callbacks::call(&CALLBACKS, id, ());
//!         });
//!
//!         m.register("setTimeout", r#"
//!             function(id, ms) {
//!                 var done = this.callbacks.done;
//!
//!                 setTimeout(function() {
//!                     done(id);
//!                 }, ms);
//!             }
//!         "#);
//!
//!         TestModule(m)
//!     }
//! }
//!
//! stasis! {{
//!     const DELAY: u32 = 1000;
//!
//!     let id = CALLBACKS
//!         .lock()
//!         .create();
//!
//!     CALLBACKS
//!         .lock()
//!         .register(id, move || {
//!             // Our return value is the unit type.
//!             let value: () = CALLBACKS
//!                 .lock()
//!                 .get(id)
//!                 .unwrap();
//!
//!             console::log(format!("Timeout finished: {:?}", value));
//!         });
//!
//!     // This will print "Timeout finished" after 1000 milliseconds.
//!     let () = unsafe {
//!         MODULE
//!             .lock()
//!             .0
//!             .call("setTimeout", (id, DELAY))
//!     };
//! }}
//! ```

use std::collections::HashMap;

use global::Global;

/// A reference to a registered callback.
///
/// This callback may be waiting to be called, or it may have already been
/// called.
#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CallbackId(u32);

/// A callback manager for asynchronous JavaScript functions.
pub struct Callbacks<T> {
    counter: u32,
    map: HashMap<CallbackId, Callback<T>>,
}

/// Manual impl to avoid the requirement of `T: Default`.
impl<T> Default for Callbacks<T> {
    fn default() -> Self {
        Self {
            counter: 0,
            map: HashMap::new(),
        }
    }
}

enum Callback<T> {
    Waiting(Box<FnMut()>),
    Ready(T),
}

impl<T: 'static> Callbacks<T> {
    /// Fetch a new callback ID.
    pub fn create(&mut self) -> CallbackId {
        let id = CallbackId(self.counter);
        self.counter += 1;
        id
    }

    /// Register a callback to be called.
    ///
    /// This will overwrite any existing callback.
    pub fn register<F>(&mut self, id: CallbackId, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        // We can't construct a Box<FnOnce> at the time of writing, so we create
        // a wrapper closure.
        let mut opt = Some(f);

        self.map.insert(id, Callback::Waiting(Box::new(move || {
            let f = opt
                .take()
                .unwrap();

            f()
        })));
    }

    /// Call a registered callback and assign a value.
    ///
    /// This will return true if the callback was not already called.
    pub fn call(global_self: &'static Global<Self>, id: CallbackId, t: T) -> bool {
        let mut lock = global_self.lock();

        // Insert the success value.
        let callback = lock.map.insert(id, Callback::Ready(t));

        // Important: this will prevent panics while locking twice.
        drop(lock);

        let (reinsert, success) = match callback {
            Some(Callback::Waiting(mut f)) => {
                f();
                (None, true)
            }

            other => (other, false),
        };

        if let Some(t) = reinsert {
            global_self
                .lock()
                .map
                .insert(id, t);
        }

        success
    }

    /// Get the return value of a callback.
    ///
    /// This will retrieve the return value of a callback if it is available.
    pub fn get(&mut self, id: CallbackId) -> Option<T> {
        match self.map.remove(&id) {
            Some(Callback::Ready(r)) => Some(r),
            Some(Callback::Waiting(f)) => {
                // Re-insert.
                self.map.insert(id, Callback::Waiting(f));
                None
            }
            None => None,
        }
    }
}
