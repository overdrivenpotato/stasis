use std::mem;

extern {
    pub fn __stasis_module_create() -> u32;
    pub fn __stasis_register(ptr: *mut u8, len: usize);
    pub fn __stasis_call(ptr: *mut u8, len: usize) -> *mut u8;
}

pub fn alloc(size: usize) -> *mut u8 {
    let mut vec = Vec::with_capacity(size as usize);
    let ptr = vec.as_mut_ptr();
    mem::forget(vec);
    ptr
}

pub unsafe fn dealloc(ptr: u32, length: u32) {
    let ptr = ptr as *mut u8;
    let length = length as usize;

    drop(Vec::from_raw_parts(ptr, length, length));
}
