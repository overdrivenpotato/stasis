//! Tutorial 3: Global Module
//!
//! Creating a module every time we need to call a function is expensive.
//! Stasis provides a `Global` type that allows users to cache their modules.
//! Let's take the module from the last tutorial and cache it as an example.
//!
//! Refresher:
//!
//! ```rust,no_run
//! #[macro_use] extern crate stasis;
//! use stasis::{console, Module};
//!
//! stasis! {{
//!     let module = Module::new();
//!
//!     module.register("random", r#"
//!         function() {
//!             return Math.random();
//!         }
//!     "#);
//!
//!     let n: f32 = unsafe {
//!         module.call("random", ())
//!     };
//!
//!     console::log(n);
//! }}
//! ```
//!
//! ## `struct Random`
//!
//! The first thing we are going to do is create a newtype wrapper around
//! `Module`. We will also implement `Default` and move the registration into
//! this impl:
//!
//! ```rust,no_run
//! # #[macro_use] extern crate stasis;
//! # use stasis::{console, Module};
//! // ...
//!
//! struct Random(Module);
//!
//! impl Default for Random {
//!     fn default() -> Random {
//!         let module = Module::new();
//!
//!         module.register("random", r#"
//!             function() {
//!                 return Math.random();
//!             }
//!         "#);
//!
//!         Random(module)
//!     }
//! }
//!
//! // ...
//! # stasis! {{}}
//! ```
//!
//! ## `static MODULE`
//!
//! Now that we have a struct that implements `Default`, we will create a static
//! global with the `Global` type.
//!
//! ```rust,no_run
//! # #[macro_use] extern crate stasis;
//! // ...
//!
//! // We import `Global` here
//! use stasis::{console, Module, Global};
//!
//! static MODULE: Global<Random> = Global::INIT;
//!
//! # struct Random(Module);
//! # impl Default for Random {
//! #     fn default() -> Random {
//! #         let module = Module::new();
//! #         module.register("random", r#"
//! #             function() {
//! #                 return Math.random();
//! #             }
//! #         "#);
//! #         Random(module)
//! #     }
//! # }
//! // ...
//! # stasis! {{}}
//! ```
//!
//! ## `fn random()`
//!
//! Finally, we can use this global value to write a wrapper function:
//!
//! ```rust,no_run
//! # #[macro_use] extern crate stasis;
//! # use stasis::{console, Module, Global};
//! # static MODULE: Global<Random> = Global::INIT;
//! // ...
//!
//! pub fn random() -> f32 {
//!     unsafe {
//!         MODULE
//!             .lock()
//!             // We get the actual module here, remember this is a wrapper
//!             // type.
//!             .0
//!             .call("random", ())
//!     }
//! }
//!
//! // ...
//! # struct Random(Module);
//! # impl Default for Random {
//! #     fn default() -> Random {
//! #         let module = Module::new();
//! #         module.register("random", r#"
//! #             function() {
//! #                 return Math.random();
//! #             }
//! #         "#);
//! #         Random(module)
//! #     }
//! # }
//! # stasis! {{}}
//! ```
//!
//! ## Complete example
//!
//! ```rust,no_run
//! #[macro_use] extern crate stasis;
//! use stasis::{console, Module, Global};
//!
//! static MODULE: Global<Random> = Global::INIT;
//!
//! struct Random(Module);
//!
//! impl Default for Random {
//!     fn default() -> Random {
//!         let module = Module::new();
//!
//!         module.register("random", r#"
//!             function() {
//!                 return Math.random();
//!             }
//!         "#);
//!
//!         Random(module)
//!     }
//! }
//!
//! pub fn random() -> f32 {
//!     unsafe {
//!         MODULE
//!             .lock()
//!             // We get the actual module here, remember this is a wrapper
//!             // type.
//!             .0
//!             .call("random", ())
//!     }
//! }
//!
//! stasis! {{
//!     console::log(random());
//! }}
//! ```
