use std::sync::{Arc, Mutex};

use serde_json;
use serde::{Serialize, Deserialize};

use internal_callbacks;
use data::Pair;

extern {
    /// The stasis call interface.
    ///
    /// This takes an opcode and 2 arguments, returning a value. The opcodes
    /// correspond to various functions:
    ///
    /// 0: Register internal callback handler
    /// 1: Create module
    /// 2: Register function
    /// 3: Register callback
    /// 4: Call function
    fn __stasis_call(op: u32, a: u32, b: u32) -> u32;
}

mod opcode {
    pub const REGISTER_STASIS_CB: u32 = 0;
    pub const CREATE_MODULE: u32 = 1;
    pub const REGISTER_FN: u32 = 2;
    pub const REGISTER_CB: u32 = 3;
    pub const CALL_FN: u32 = 4;
}

lazy_static! {
    static ref STASIS_CALLBACK_REGISTERED: Arc<Mutex<bool>> = {
        Arc::new(Mutex::new(false))
    };
}

type StasisCallback = extern fn(op: u32, a: u32, b: u32) -> *mut u8;

pub fn register_stasis_callback(f: StasisCallback) {
    unsafe {
        __stasis_call(opcode::REGISTER_STASIS_CB, f as u32, 0);
    }
}

pub fn create_module() -> u32 {
    // In stasis, a module must always be created before anything else can be
    // done. With this reasoning, we can place the callback registration code
    // here.

    let mut guard = STASIS_CALLBACK_REGISTERED.lock().unwrap();

    if !*guard {
        register_stasis_callback(::incoming::incoming);
        *guard = true;
    }

    drop(guard);

    unsafe {
        __stasis_call(opcode::CREATE_MODULE, 0, 0)
    }
}

pub fn register_fn(module_id: u32, name: &str, code: &str) {
    #[derive(Serialize)]
    struct RegisterFn<'a, 'b> {
        // TODO: Rename this to module_id?
        id: u32,
        name: &'a str,
        code: &'b str,
    }

    let data = RegisterFn { id: module_id, name, code };

    let Pair { ptr, len } = Pair::serialize(&data).unwrap();

    unsafe {
        __stasis_call(opcode::REGISTER_FN, ptr as u32, len as u32);
    }
}

/// Register a callback.
///
/// The function must be `Sync` as it can be recursively called. This prevents
/// a deadlock from occurring.
pub fn register_callback<F, A, R>(module_id: u32, name: &str, f: F)
where
    F: 'static + Send + Sync + Fn(A) -> R,
    A: for<'a> Deserialize<'a>,
    R: Serialize,
{
    #[derive(Serialize)]
    struct RegisterCallback<'a> {
        module: u32,
        callback: u32,
        name: &'a str,
    }

    let callback_id = internal_callbacks::register(f);

    let data = RegisterCallback {
        module: module_id,
        callback: callback_id,
        name,
    };

    let Pair { ptr, len } = Pair::serialize(&data).unwrap();

    unsafe {
        __stasis_call(opcode::REGISTER_CB, ptr as u32, len as u32);
    }
}

pub fn call<T, R>(module_id: u32, name: &str, args: T) -> R
where
    T: Serialize,
    R: for<'a> Deserialize<'a>,
{
    #[derive(Serialize)]
    struct Call<'a, T> {
        id: u32,
        name: &'a str,
        args: T,
    }

    let call = Call {
        id: module_id,
        name,
        args,
    };

    let Pair { ptr, len } = match Pair::serialize(call) {
        Ok(pair) => pair,
        Err(e) => panic!("Failed to serialize arguments: {}", e),
    };

    let ret = unsafe {
        __stasis_call(opcode::CALL_FN, ptr as u32, len as u32) as *mut u8
    };

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
