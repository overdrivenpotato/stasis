extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::mem;

use serde::{Serialize, Deserialize};

#[path = "rt.rs"]
#[doc(hidden)]
/// The runtime module.
pub mod __rt;

/// Internal callback handling.
///
/// This is not the same as the `callbacks` module, that is for users of this
/// library to create their own callbacks. This module handles the raw callback
/// interface.
mod rt_callbacks;

pub mod global;
pub mod callbacks;
pub mod tutorial;

pub use global::{Global, GlobalLock};
pub use callbacks::Callbacks;

#[macro_export]
/// Declare the main function.
///
/// This is equivalent to `fn main() { ... }`, however it also declares hooks
/// necessary for stasis to load the binary.
macro_rules! stasis {
    ($body:block) => {
        #[allow(unused)]
        #[doc(hidden)]
        fn main() {}

        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn __stasis_entrypoint() {
            // Mount the crate functions first.
            $crate::load();

            fn entrypoint() $body
            entrypoint();
        }

        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn __stasis_alloc(n: usize) -> *mut u8 {
            $crate::__rt::alloc(n)
        }

        #[no_mangle]
        #[doc(hidden)]
        pub unsafe extern "C" fn __stasis_dealloc(ptr: u32, len: u32) {
            $crate::__rt::dealloc(ptr, len)
        }

        #[no_mangle]
        #[doc(hidden)]
        pub unsafe extern "C" fn __stasis_callback(ptr: *mut u8, len: usize) -> *mut u8 {
            $crate::__rt::callback(ptr, len)
        }
    }
}

/// A unique module instance.
#[derive(Clone, Copy)]
pub struct Module {
    id: u32,
}

/// A WebAssembly-friendly fat pointer.
#[derive(Debug)]
struct Pair {
    pub ptr: *mut u8,
    pub len: usize,
}

impl Pair {
    fn serialize<T>(t: T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        serde_json::to_string(&t)
            .map(|s| s.into())
    }

    unsafe fn from_u8_mut_ptr(src: *mut u8) -> Self {
        let mut len: u32 = 0;
        let mut ptr: u32 = 0;

        ptr += *src as u32;
        ptr += (*src.offset(1) as u32) << 8;
        ptr += (*src.offset(2) as u32) << 16;
        ptr += (*src.offset(3) as u32) << 24;

        len += *src.offset(4) as u32;
        len += (*src.offset(5) as u32) << 8;
        len += (*src.offset(6) as u32) << 16;
        len += (*src.offset(7) as u32) << 24;

        // Deallocate the fat pointer.
        drop(Vec::from_raw_parts(src, 8, 8));

        Self {
            ptr: ptr as *mut u8,
            len: len as usize,
        }
    }

    unsafe fn into_string(self) -> String {
        String::from_raw_parts(self.ptr, self.len, self.len)
    }
}

impl From<String> for Pair {
    fn from(s: String) -> Self {
        let mut bytes: Vec<u8> = s.into();
        bytes.shrink_to_fit();

        let ptr = bytes.as_mut_ptr();
        let len = bytes.len();

        mem::forget(bytes);

        Self { ptr, len }
    }
}

impl Into<*mut u8> for Pair {
    fn into(self) -> *mut u8 {
        let Self { ptr, len } = self;

        let mut bytes: Vec<u8> = Vec::with_capacity(8);

        for _ in 0..8 {
            bytes.push(0);
        }

        let ptr = ptr as u32;
        let len = len as u32;

        bytes[0] = (ptr & 0xFF) as u8;
        bytes[1] = ((ptr & 0xFF00) >> 8) as u8;
        bytes[2] = ((ptr & 0xFF0000) >> 16) as u8;
        bytes[3] = ((ptr & 0xFF000000) >> 24) as u8;

        bytes[4] = (len & 0xFF) as u8;
        bytes[5] = ((len & 0xFF00) >> 8) as u8;
        bytes[6] = ((len & 0xFF0000) >> 16) as u8;
        bytes[7] = ((len & 0xFF000000) >> 24) as u8;

        let ret = bytes.as_mut_ptr();

        mem::forget(bytes);

        ret
    }
}

impl Module {
    pub fn new() -> Self {
        Self {
            id: unsafe { ::__rt::__stasis_module_create() }
        }
    }

    pub fn register(&self, name: &str, code: &str) {
        #[derive(Serialize)]
        struct Register<'a, 'b> {
            id: u32,
            name: &'a str,
            code: &'b str,
        }

        let reg = Register {
            id: self.id,
            name,
            code,
        };

        let Pair { ptr, len } = Pair::serialize(&reg).unwrap();

        // Unsafe due to FFI call.
        unsafe {
            ::__rt::__stasis_register(ptr, len);
        }
    }

    pub fn register_callback<F, A, R>(&self, name: &str, f: F)
    where
        F: 'static + Fn(A) -> R,
        A: for<'a> Deserialize<'a>,
        R: Serialize,
    {
        #[derive(Serialize)]
        struct RegisterCallback<'a> {
            module: u32,
            callback: u32,
            name: &'a str,
        }

        let id = rt_callbacks::register(f);

        let reg = RegisterCallback {
            module: self.id,
            callback: id,
            name,
        };

        let Pair { ptr, len } = Pair::serialize(&reg).unwrap();

        // Unsafe due to FFI call.
        unsafe {
            ::__rt::__stasis_register_callback(ptr, len);
        }
    }

    pub fn call<T, R>(&self, name: &str, args: T) -> R
    where
        T: Serialize,
        R: for<'a> Deserialize<'a>
    {
        #[derive(Serialize)]
        struct Call<'a, T> {
            id: u32,
            name: &'a str,
            args: T,
        }

        let call = Call {
            id: self.id,
            name,
            args,
        };

        let Pair { ptr, len } = match Pair::serialize(call) {
            Ok(pair) => pair,
            Err(e) => panic!("Failed to serialize arguments: {}", e),
        };

        // This unsafety is due to an FFI call.
        let ret = unsafe { ::__rt::__stasis_call(ptr, len) };

        let value = if ret.is_null() {
            "null".to_owned()
        } else {
            // `ret` is given to us by the FFI function so we must assume it is
            // safe.
            unsafe {
                Pair::from_u8_mut_ptr(ret).into_string()
            }
        };

        match serde_json::from_str(&value) {
            Ok(v) => v,
            Err(e) => {
                panic!(
                    "STASIS: Failed to deserialize return value.\n\
                     Given '{}'\n\
                     Error {:?}",
                    value,
                    e
                )
            }
        }
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

/// Setup the panic handler.
fn setup_panic() {
    use std::panic;

    panic::set_hook(Box::new(|info| {
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

#[doc(hidden)]
pub fn load() {
    setup_panic();
}
