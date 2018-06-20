//! Easy handling of JavaScript callbacks.
//!
//! This module is designed to work well with [`global`].
//!
//! ```rust,no_run
//! #[macro_use] extern crate stasis;
//!
//! use stasis::{console, Module, Global, Callbacks};
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
//!         m.register_callback("done", |id: u32| {
//!             Callbacks::call(&CALLBACKS, id, ());
//!         });
//!
//!         m.register("rand", r#"
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
//!     const TEST_ID: u32 = 0;
//!     const DELAY: u32 = 1000;
//!
//!     CALLBACKS
//!         .lock()
//!         .register(TEST_ID, || {
//!             console::log("Timeout finished.");
//!         });
//!
//!     // This will print "Timeout finished" after 1000 milliseconds.
//!     let () = unsafe {
//!         MODULE
//!             .lock()
//!             .0
//!             .call("rand", (TEST_ID, DELAY))
//!     };
//! }}
//! ```

use std::collections::HashMap;

use global::Global;

/// A callback manager for asynchronous JavaScript functions.
pub struct Callbacks<T> {
    map: HashMap<u32, Callback<T>>,
}

impl<T> Default for Callbacks<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

enum Callback<T> {
    Waiting(Box<FnMut()>),
    Ready(T),
}

impl<T: 'static> Callbacks<T> {
    /// Register a callback to be called.
    ///
    /// This will overwrite any existing callback with the same ID.
    pub fn register<F>(&mut self, id: u32, f: F)
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

    /// Return a callback with a global handle.
    ///
    /// This will return true if there was a registered callback that was
    /// waiting to be called.
    pub fn call(global_self: &'static Global<Self>, id: u32, t: T) -> bool {
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
    pub fn get(&mut self, id: u32) -> Option<T> {
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
