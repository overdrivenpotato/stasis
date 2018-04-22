extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::mem;

use serde::{Serialize, Deserialize};

#[path = "./rt.rs"]
#[doc(hidden)]
/// The runtime module.
pub mod __rt;

#[macro_export]
/// A stasis entry point.
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
        pub unsafe extern "C" fn __stasis_dealloc(ptr: u32, length: u32) {
            $crate::__rt::dealloc(ptr, length)
        }
    }
}

struct Module {
    id: u32,
}

impl Module {
    fn new() -> Self {
        Self {
            id: unsafe { ::__rt::__stasis_module_create() }
        }
    }

    fn register<'a, 'b, 'c>(&'a mut self, name: &'b str, code: &'c str) {
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

        let json = serde_json::to_string(&reg).unwrap();
        let mut bytes: Vec<u8> = json.into();
        bytes.shrink_to_fit();

        unsafe {
            ::__rt::__stasis_register(bytes.as_mut_ptr(), bytes.len());
        }

        mem::forget(bytes);
    }

    unsafe fn call<T, R>(&self, name: &str, args: T) -> R
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

        let json = serde_json::to_string(&call).unwrap();
        let mut bytes: Vec<u8> = json.into();
        bytes.shrink_to_fit();

        let ret = ::__rt::__stasis_call(bytes.as_mut_ptr(), bytes.len());
        mem::forget(bytes);

        let value = if ret.is_null() {
            "null".to_owned()
        } else {
            let mut len: u32 = 0;
            let mut ptr: u32 = 0;

            ptr += *ret as u32;
            ptr += (*ret.offset(1) as u32) << 8;
            ptr += (*ret.offset(2) as u32) << 16;
            ptr += (*ret.offset(3) as u32) << 24;

            len += *ret.offset(4) as u32;
            len += (*ret.offset(5) as u32) << 8;
            len += (*ret.offset(6) as u32) << 16;
            len += (*ret.offset(7) as u32) << 24;

            // Deallocate the fat pointer.
            ::__rt::dealloc(ret as u32, 8);

            let ptr = ptr as *mut u8;
            let len = len as usize;

            // This will be deallocated when it is dropped.
            String::from_raw_parts(ptr, len, len)
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

static mut STASIS_PRELUDE: Option<Module> = None;

fn prelude() -> &'static mut Module {
    unsafe { STASIS_PRELUDE.as_mut().unwrap() }
}

pub fn alert<T>(t: T) where T: Serialize {
    unsafe {
        prelude().call("alert", t)
    }
}

pub struct Performance {}

impl Performance {
    pub fn new() -> Self {
        let () = unsafe {
            prelude().call("performance.start", ())
        };

        Performance {}
    }

    pub fn stop(self) -> u32 {
        unsafe {
            prelude().call("performance.end", ())
        }
    }
}

pub mod console {
    use serde::Serialize;

    use super::prelude;

    pub fn log<T>(t: T) where T: Serialize {
        unsafe {
            prelude().call("console.log", t)
        }
    }

    pub fn error<T>(t: T) where T: Serialize {
        unsafe {
            prelude().call("console.error", t)
        }
    }

    pub fn warn<T>(t: T) where T: Serialize {
        unsafe {
            prelude().call("console.warn", t)
        }
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
            .unwrap_or("Panic occured in unknown location".to_owned());

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

    let mut m = Module::new();

    // Common global functions.
    m.register("console.log", "console.log");
    m.register("console.error", "console.error");
    m.register("console.warn", "console.warn");
    m.register("alert", r#"
        function() {
            alert.apply(window, arguments);
        }
    "#);
    m.register("performance.start", r#"
        function() {
            this.data.performance = performance.now();
        }
    "#);
    m.register("performance.end", r#"
        function() {
            var start = this.data.performance;
            var end = performance.now();

            // Delete the performance data.
            delete this.data.performance;

            return end - start;
        }
    "#);

    // Assign the global prelude.
    unsafe {
        STASIS_PRELUDE = Some(m);
    }
}
