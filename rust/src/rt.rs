//! Runtime hooks for the WebAssembly binary.
//!
//! The contents of this module are *not guaranteed to be stable!*

use std::mem;

use serde_json;

use super::Pair;
use rt_callbacks;

extern {
    pub fn __stasis_module_create() -> u32;
    pub fn __stasis_register(ptr: *mut u8, len: usize);
    pub fn __stasis_register_callback(ptr: *mut u8, len: usize);
    pub fn __stasis_call(ptr: *mut u8, len: usize) -> *mut u8;
}

pub fn alloc(size: usize) -> *mut u8 {
    let mut vec = Vec::with_capacity(size as usize);
    let ptr = vec.as_mut_ptr();
    mem::forget(vec);
    ptr
}

pub unsafe fn dealloc(ptr: u32, len: u32) {
    let ptr = ptr as *mut u8;
    let len = len as usize;

    drop(Vec::from_raw_parts(ptr, len, len));
}

pub unsafe fn callback(ptr: *mut u8, len: usize) -> *mut u8 {
    #[derive(Deserialize)]
    struct Callback {
        id: u32,
        ptr: u32,
        len: u32,
    }

    let json = String::from_raw_parts(ptr, len, len);

    let Callback { ptr, len, id } = serde_json::from_str(&json).unwrap();

    let ptr = ptr as *mut u8;
    let len = len as usize;

    let params = String::from_raw_parts(ptr, len, len);

    let ret = rt_callbacks::call(id, params);

    match ret {
        // Use `Pair` as an intermediate format.
        Some(s) => Pair::from(s).into(),
        None => 0 as *mut u8,
    }
}
