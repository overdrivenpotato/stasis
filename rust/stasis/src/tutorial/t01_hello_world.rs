//! Tutorial 1: Hello World!
//!
//! Let's start out by making a hello world application. The first thing we will
//! do is create our project.
//!
//! ```sh
//! $ cargo new hello_world
//! $ cd hello_world
//! $ tree
//! .
//! ├── Cargo.toml
//! └── src
//!     └── main.rs
//!
//! 1 directory, 2 files
//! ```
//!
//! Add to your `Cargo.toml` dependencies section:
//!
//! ```toml
//! [package]
//! name = "hello_world"
//! version = "0.1.0"
//! authors = ["Your Name <you@yoursite.com>"]
//!
//! [dependencies]
//! stasis = "1.0"
//! ```
//!
//! Now we can begin! Open up main.rs and import the crate:
//!
//! ```rust,no_run
//! #[macro_use] extern crate stasis;
//!
//! fn main() {}
//! ```
//!
//! Unlike a normal rust application, `stasis` requires the use of the `stasis!`
//! macro for application entrypoint. This is equivalent to `fn main() {}` like
//! we are used to.
//!
//! ```rust,no_run
//! #[macro_use] extern crate stasis;
//!
//! stasis! {{
//!     // Your code goes here...
//! }}
//! ```
//!
//! Let's import the `console` module from stasis and print our message!
//!
//! ```rust,no_run
//! #[macro_use] extern crate stasis;
//!
//! use stasis::console;
//!
//! stasis! {{
//!     console::log("Hello World!");
//! }}
//! ```
//!
//! Voila! You've written your first stasis application.
