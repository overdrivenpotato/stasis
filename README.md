# Stasis

A complete runtime for rust applications in the browser.

* No npm
* No webpack
* No bindgen
* No cargo web
* No javascript build system
* No build step at all
* Full web applications in pure rust

```rust
#[macro_use] extern crate stasis;

use stasis::{console, Module};

// The main function.
stasis! {{
    console::log("Hello World!");

    // Create a new Stasis module.
    let mut module = Module::new();

    // Register a javascript function.
    module.register("add", r#"
        function (a, b) {
            return a + b;
        }
    "#);

    // Call the registered javascript function.
    let added = unsafe {
        module.call("add", (1, 2))
    };

    // Functions can accept multiple arguments in the form of tuples.
    console::log(("Added 1 and 2 to get:", added));

    // The above line is equivalent to the following JS code:
    // console.log('Added 1 and 2 to get:', added)
}}
```

This library is intended to be used as a base for other libraries. The main
goals of this project are **high performance**, **small size**, and
an **opinionated design**.

## Performance

Registering a function compiles it immediately with the use of `eval`. This
allows the browser JS engine to optimize on a per-function basis, with no
interpreter overhead at the time of a call.

TODO: Benchmarks

## Why only rust?

The Stasis runtime hands a small WebAssembly environment to the binary. This
environment is language agnostic. I currently have no plans to create bindings
to languages other than rust, however feel free to open a pull request if you
would like to do so yourself.

## Runtime API

Stasis uses JSON as a transfer format between JS and WebAssembly. This may be
changed in the future, however Stasis will make sure to be backwards compatible
from both the library *and* runtime point of view.
