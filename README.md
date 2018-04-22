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

This library is intended to be used as a base for other libraries. The goal of
this project is to provide the ability to completely eject from the javascript
ecosystem, while also allowing interop with existing javascript applications.

**A complete example can be found at the bottom of this document.**

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

# Embedding Stasis into an existing project

Stasis is designed to be easily embeddable in existing projects. The runtime is
not global, so you can have many instances if needed. The `stasis` package on
npm exports a function which accepts a path to the binary.

Example:

```javascript
import load from 'stasis'

load('/code/bundle1.wasm')
  .then(() => {
    console.log('Bundle 1\'s main() has finished running!')
  })

load('/code/bundle2.wasm')
  .then(() => {
    console.log('Bundle 2\'s main() has finished running!')
  })

// ...
```

# Complete Standalone Example

This is a complete application that will print `Hello world!` to the browser
console. To run it in chrome, open a webserver at the project root and navigate
to `index.html`. If you are using firefox, you can simply open the `index.html`
file directly.

```
.
├── Cargo.toml
├── index.html
└── src
    └── main.rs
```

`Cargo.toml`

```toml
[package]
name = "stasis-test"
version = "0.1.0"
authors = ["Marko Mijalkovic <marko.mijalkovic97@gmail.com>"]

[dependencies]
stasis = "0.1.0-alpha"
```

`src/main.rs`

```rust
#[macro_use] extern crate stasis;

use stasis::console;

stasis! {{
    console::log("Hello world!");
}}
```

`index.html`

NOTE: This file only needs to be written once. This is only necessary to allow
Stasis to bootstrap the WebAssembly binary. In a full application, this file
would remain identical.

```html
<!DOCTYPE html>
<html>
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width" />
        <title>test</title>
        <script
            id="stasis"
            src="http://bundle.run/stasis@0.1.0-alpha.3/dist/stasis.min.js"
            type="text/javascript"
            data-binary="target/wasm32-unknown-unknown/release/stasis-test.wasm"
        ></script>
    </head>
    <body></body>
</html>
```
