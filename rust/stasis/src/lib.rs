/// This crate is recommended as the way to implement module memoization.
pub extern crate global;

extern crate futures_v01x;
extern crate futures_v02x;
extern crate once_nonstatic;
extern crate serde;
#[macro_use] extern crate serde_derive;

/// This must be public to be accessed via the `stasis!` macro. There is a
/// `#[doc(hidden)]` attribute on here as this should never be used by a user of
/// the library directly.
#[doc(hidden)]
pub extern crate stasis_internals;

use global::Global;
use serde::{Serialize, Deserialize};

pub mod callbacks;
pub mod tutorial;
pub mod futures;

/// A unique module instance.
#[derive(Clone, Copy)]
pub struct Module {
    id: u32,
}

impl Module {
    pub fn new() -> Self {
        Self {
            id: stasis_internals::outgoing::create_module(),
        }
    }

    pub fn register(&self, name: &str, code: &str) {
        stasis_internals::outgoing::register_fn(self.id, name, code);
    }

    pub fn register_callback<F, A, R>(&self, name: &str, f: F)
    where
        F: 'static + Send + Sync + Fn(A) -> R,
        A: for<'a> Deserialize<'a>,
        R: Serialize,
    {
        stasis_internals::outgoing::register_callback(self.id, name, f);
    }

    pub fn call<T, R>(&self, name: &str, args: T) -> R
    where
        T: Serialize,
        R: for<'a> Deserialize<'a>
    {
        stasis_internals::outgoing::call(self.id, name, args)
    }
}

/// Prelude implementation.
struct Prelude(Module);

/// Prelude instance.
static PRELUDE: Global<Prelude> = Global::INIT;

impl Default for Prelude {
    fn default() -> Self {
        let m = Module::new();

        // Common global functions.
        m.register("console.log", "console.log");
        m.register("console.error", "console.error");
        m.register("console.warn", "console.warn");
        m.register("alert", r#"
            function(s) {
                window.alert(s);
            }
        "#);

        Prelude(m)
    }
}

/// Browser alert.
///
/// Equivalent to `window.alert(...)`.
pub fn alert<T>(t: T) where T: ToString {
    PRELUDE.lock().0.call("alert", t.to_string())
}

pub mod console {
    //! The browser `console` interface.

    use serde::Serialize;

    use super::PRELUDE;

    /// Log a message to the console.
    ///
    /// This can be called with multiple arguments in a tuple or array.
    pub fn log<T>(t: T) where T: Serialize {
        PRELUDE.lock().0.call("console.log", t)
    }

    /// Log an error to the console.
    ///
    /// This can be called with multiple arguments in a tuple or array.
    pub fn error<T>(t: T) where T: Serialize {
        PRELUDE.lock().0.call("console.error", t)
    }

    /// Log a warning to the console.
    ///
    /// This can be called with multiple arguments in a tuple or array.
    pub fn warn<T>(t: T) where T: Serialize {
        PRELUDE.lock().0.call("console.warn", t)
    }
}

/// Setup a panic handler.
///
/// This sends all panics to the console.
pub fn setup_panic() {
    std::panic::set_hook(Box::new(|info| {
        let message = info
            .location()
            .map(|loc| {
                format!("Panic!\nLine {} in {}", loc.line(), loc.file())
            })
            .unwrap_or("Panic in unknown location".to_owned());

        let payload = info
            .payload()
            .downcast_ref::<String>()
            .map(|s| s.clone())
            .or_else(|| {
                info.payload()
                    .downcast_ref::<&str>()
                    .map(|&s| s.to_owned())
            })
            .unwrap_or("No panic info.".to_owned());

        let s = format!("{}:\n\n{}", message, payload);

        console::error(&s);
    }));
}
