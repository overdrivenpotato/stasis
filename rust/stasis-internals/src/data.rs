//! Data reading and writing.

use std::mem;

use serde_json;
use serde::{Serialize};

/// Little Endian read of `u32`.
///
/// # Panics
///
/// This function will panic if the given slice is not exactly 4 bytes long.
pub fn read_u32(ptr: &[u8]) -> u32 {
    assert_eq!(ptr.len(), 4);

       (ptr[0] as u32)
    + ((ptr[1] as u32) << 8)
    + ((ptr[2] as u32) << 16)
    + ((ptr[3] as u32) << 24)
}

/// Little Endian write for `u32`.
///
/// # Panics
///
/// This function will panic if the given slice is not exactly 4 bytes long.
pub fn write_u32(ptr: &mut [u8], n: u32) {
    assert_eq!(ptr.len(), 4);

    ptr[0] = (n & 0xFF) as u8;
    ptr[1] = ((n & 0xFF00) >> 8) as u8;
    ptr[2] = ((n & 0xFF0000) >> 16) as u8;
    ptr[3] = ((n & 0xFF000000) >> 24) as u8;
}

/// A WebAssembly-friendly fat pointer.
#[derive(Debug)]
pub struct Pair {
    pub ptr: *mut u8,
    pub len: usize,
}

impl Pair {
    pub fn serialize<T>(t: T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        serde_json::to_string(&t)
            .map(|s| s.into())
    }

    pub unsafe fn from_u8_mut_ptr(src: *mut u8) -> Self {
        let bytes = Vec::from_raw_parts(src, 8, 8);

        let ptr = read_u32(&bytes[0..4]);
        let len = read_u32(&bytes[4..8]);

        // Deallocate the fat pointer. This call is not actually needed but
        // helps illustrate what needs to happen here.
        drop(bytes);

        Self {
            ptr: ptr as *mut u8,
            len: len as usize,
        }
    }

    pub unsafe fn into_string(self) -> String {
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

        let mut bytes = Vec::with_capacity(8);

        for _ in 0..8 {
            bytes.push(0);
        }

        write_u32(&mut bytes[0..4], ptr as u32);
        write_u32(&mut bytes[4..8], len as u32);

        let ret = bytes.as_mut_ptr();

        mem::forget(bytes);

        ret
    }
}
