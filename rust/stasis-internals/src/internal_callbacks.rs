use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json;
use serde::{Serialize, Deserialize};

lazy_static! {
    static ref HANDLER: Mutex<Callbacks> = Default::default();
}

/// A registered callback.
type Callback = Arc<Box<Fn(String) -> String + Send + Sync>>;

/// A global callback list.
#[derive(Default)]
struct Callbacks {
    current: u32,
    registered: HashMap<u32, Callback>,
}

impl Callbacks {
    fn register<F>(&mut self, f: F) -> u32
    where
        F: 'static + Send + Sync + Fn(String) -> String,
    {
        let id = self.current;
        self.current += 1;

        self.registered.insert(id, Arc::new(Box::new(f)));

        id
    }
}

/// Register a callback.
///
/// The function must be `Sync` as it can be recursively called.
pub fn register<F, A, R>(f: F) -> u32
where
    F: 'static + Send + Sync + Fn(A) -> R,
    A: for<'a> Deserialize<'a>,
    R: Serialize,
{
    let mut guard = HANDLER.lock().unwrap();

    guard.register(move |input| {
        // This is guaranteed to never fail by the user.
        let input = match serde_json::from_str(&input) {
            Ok(o) => o,
            Err(e) => {
                panic!(
                    "Stasis: Failed to deserialize argument to callback.\n\
                     Error: {}",
                    e,
                )
            }
        };

        let output = f(input);

        // This should also never fail.
        serde_json::to_string(&output).unwrap()
    })
}

pub fn call(id: u32, args: String) -> Option<String> {
    let guard = HANDLER.lock().unwrap();

    let f = guard.registered
        .get(&id)
        .cloned()
        .expect(
            "FATAL: Failed to find callback. Make sure to register all\
             callbacks."
        );

    // Important: A callback may be called recursively.
    drop(guard);

    match f(args) {
        // Optimize for the null pointer.
        ref s if s == "null" => None,
        s => Some(s),
    }
}
