//! Tutorial 2: JavaScript Functions
//!
//! ## Creating a `Module`
//!
//! We're going to create a wrapper around `Math.random()` in JavaScript.
//! For this tutorial we will be building off of the previous project. We're
//! going to start off by importing and initializing a `Module` instance.
//!
//! ```rust,no_run
//! #[macro_use] extern crate stasis;
//!
//! use stasis::Module;
//!
//! stasis! {{
//!     let module = Module::new();
//! }}
//! ```
//!
//! ## Registering JavaScript code
//!
//! To call into JavaScript code, we must first register a JavaScript function.
//!
//! ```rust,no_run
//! # #[macro_use] extern crate stasis;
//! # use stasis::Module;
//! // ...
//! stasis! {{
//!     let module = Module::new();
//!
//!     module.register("random", r#"
//!         function() {
//!             return Math.random();
//!         }
//!     "#);
//! }}
//! ```
//!
//! ## Calling JavaScript code.
//!
//! Now let's call this function. The return type must explicitly be annotated.
//!
//! ```rust,no_run
//! # #[macro_use] extern crate stasis;
//! # use stasis::{console, Module};
//! // ...
//! stasis! {{
//!     // ...
//! #     let module = Module::new();
//! #
//! #     module.register("random", r#"
//! #         function() {
//! #             return Math.random();
//! #         }
//! #     "#);
//!     let n: f32 = module.call("random", ());
//!
//!     console::log(n);
//! }}
//! ```
//!
//! This will print out our random number. The second argument given to `call`
//! is the tuple of arguments passed to the JavaScript function. In this case,
//! our function accepts no arguments so we pass in an empty tuple.
//!
//! ## Complete Example
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
//!     let n: f32 = module.call("random", ());
//!
//!     console::log(n);
//! }}
//! ```
