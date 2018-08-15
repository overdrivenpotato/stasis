//! Runtime hooks for the WebAssembly binary.
//!
//! The contents of this module are *not guaranteed to be stable!*

use std::mem;

use internal_callbacks;
use data::{self, Pair};

mod opcode {
    pub const ALLOC: u32 = 0;
    pub const DEALLOC: u32 = 1;
    pub const CALLBACK: u32 = 2;
}

pub extern fn incoming(op: u32, a: u32, b: u32) -> *mut u8 {
    use std::ptr;
    match op {
        // Allocate.
        opcode::ALLOC => {
            alloc(a as usize)
        }

        // Deallocate.
        opcode::DEALLOC => {
            unsafe {
                dealloc(a, b);
            }

            ptr::null_mut()
        }

        // Callback.
        opcode::CALLBACK => {
            unsafe {
                callback(a as *mut u8)
            }
        }

        // Unknown op code.
        _ => (-1i32) as *mut u8,
    }
}

fn alloc(size: usize) -> *mut u8 {
    let mut vec = Vec::with_capacity(size as usize);
    let ptr = vec.as_mut_ptr();
    mem::forget(vec);
    ptr
}

unsafe fn dealloc(ptr: u32, len: u32) {
    let ptr = ptr as *mut u8;
    let len = len as usize;

    drop(Vec::from_raw_parts(ptr, len, len));
}

unsafe fn callback(data: *mut u8) -> *mut u8 {
    const TRI_LEN: usize = 3 * mem::size_of::<u32>();

    let bytes = Vec::from_raw_parts(data, TRI_LEN, TRI_LEN);

    let id = data::read_u32(&bytes[0..4]);
    let ptr = data::read_u32(&bytes[4..8]) as *mut u8;
    let len = data::read_u32(&bytes[8..12]) as usize;

    let ptr = ptr as *mut u8;
    let len = len as usize;

    let params = String::from_raw_parts(ptr, len, len);

    let ret = internal_callbacks::call(id, params);

    match ret {
        // Use `Pair` as an intermediate format.
        Some(s) => Pair::from(s).into(),
        None => 0 as *mut u8,
    }
}
