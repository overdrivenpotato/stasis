# Stasis

A complete runtime for rust applications in the browser.

* No npm
* No webpack
* No bindgen
* No cargo web
* No JavaScript build system
* No build step at all
* Stable Rust
* Full web applications in pure rust

```rust
#[macro_use] extern crate stasis;

use stasis::{console, Module};

// The main function. This macro is the only necessary part of stasis.
stasis! {{
    console::log("Hello World!");

    // Create a new Stasis module.
    let mut module = Module::new();

    // Register a JavaScript function.
    module.register("add", r#"
        function (a, b) {
            return a + b;
        }
    "#);

    // Call the registered JavaScript function.
    let added: i32 = module.call("add", (1, 2));

    // Functions can accept multiple arguments in the form of tuples.
    console::log(("Added 1 and 2 to get:", added));

    // The above line is equivalent to the following JS code:
    // console.log('Added 1 and 2 to get:', added)
}}
```

This library is intended to be used as a base for other libraries. The goal of
this project is to provide the ability to completely eject from the JavaScript
ecosystem, while also allowing interop with existing JavaScript applications.

**A complete example can be found at the bottom of this document.**

# FAQ

## Why not [`stdweb`](https://github.com/koute/stdweb)?

The goal of `stdweb` is to provide Rust bindings to Web APIs while maintaining
interoperability between the two languages. Stasis takes a different approach,
it views JavaScript as something closer to assembly that you can opt-in to use.
Stasis allows you to treat JavaScript code the same way we treat `unsafe`, by
creating safe Rust-first wrappers around the JavaScript APIs offered to us in
the browser.

## What about `#![feature(wasm_syscall)]`?

WebAssembly syscalls in Rust are not currently specific to JavaScript. This
means we cannot use the API to craft JavaScript calls. Besides this, stasis aims
to work on stable rust. Eventually, if custom syscalls can be implemented, the
`stasis!` macro will be able to be removed. This is ideal as it does not force
consumers of this library to use a specific tool whether it is `stasis`,
`stdweb`, `wasm-bindgen`, or anything else.

## How does this work?

Stasis creates a map of all registered modules and functions. When calling a
registered function from WebAssembly, objects describing the request are passed
in JSON format. This may be changed in the future, however Stasis will make sure
to be backwards compatible from both the library *and* runtime point of view.

## What's the performance like?

Registering a function compiles it immediately with the use of `eval`. This
allows the browser JS engine to optimize on a per-function basis, with no
interpreter overhead at the time of a call.

For tight loops that call heavily into JavaScript, library authors are
encouraged to batch data if possible. This will minimize the overhead present
when calling JavaScript code from WebAssembly.

### TODO: Benchmarks here.

## Isn't serde a huge dependency?

When compiling with `lto = true`, the resulting .wasm binary tends to be only
slightly larger (on the order of 3kb difference). This extra bloat makes it much
easier to interlace JavaScript and rust code at the expense of a small size
increase. I don't believe this is an issue, however if you have a case for a
better solution, please let me know and I am open to changes.

## This doesn't work in node!

Stasis is not supported on node. The aim of this project is to enable rust
codebases to run on the browser. If you wish to call rust code from node, this
is already supported in both languages via their respective FFI.

## Won't unminified JavaScript increase final file size?

Gzipping the final `.wasm` binary will cut down on most of the file file size.
However yes, unminified JavaScript will be a bit larger. Because stasis
encourages only writing absolutely necessary glue code in JavaScript, realistic
differences should be small. From preliminary testing with a `fetch`
implementation, the difference between minified and unminified code was 52
bytes. If a large stasis library is causing intense bloat of the binary, the
library author is encouraged to have a `build.rs` script which can run a
JavaScript minifier first, then include the final file with `include_str!` in
the source.

### TODO: Publish a fetch example.

## Why only rust?

The Stasis runtime hands a small WebAssembly environment to the binary. This
environment is language agnostic. I currently have no plans to create bindings
to languages other than rust, however feel free to open a pull request if you
would like to do so yourself.

# Embedding Stasis into existing JavaScript

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

# Complete standalone example

This is a complete application that will print `Hello world!` to the browser
console. To run it in chrome, open a webserver at the project root and navigate
to `index.html`. If you are using firefox, you can simply open the `index.html`
file directly.

## Project structure

```
.
├── Cargo.toml
├── index.html
└── src
    └── main.rs
```

## `Cargo.toml`

```toml
[package]
name = "stasis-test"
version = "0.1.0"
authors = ["Marko Mijalkovic <marko.mijalkovic97@gmail.com>"]

[dependencies]
stasis = "0.1.0-alpha.1"
```

## `src/main.rs`

```rust
#[macro_use] extern crate stasis;

use stasis::console;

stasis! {{
    console::log("Hello world!");
}}
```

## `index.html`

NOTE: This file only needs to be written once. This is only necessary to allow
Stasis to bootstrap the WebAssembly binary. In a full application, this file
would remain identical.

```html
<!DOCTYPE html>
<html>
    <head>
        <script
            id="stasis"
            src="http://bundle.run/stasis@0.1.0-alpha.5/dist/stasis.min.js"
            type="text/javascript"
            data-binary="target/wasm32-unknown-unknown/release/stasis-test.wasm"
        ></script>
    </head>
    <body></body>
</html>
```
