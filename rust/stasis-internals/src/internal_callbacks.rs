use std::collections::HashMap;
use std::sync::Mutex;

use serde_json;
use serde::{Serialize, Deserialize};

lazy_static! {
    static ref HANDLER: Mutex<Callbacks> = Default::default();
}

/// A global callback list.
#[derive(Default)]
struct Callbacks {
    current: u32,
    registered: HashMap<u32, Box<Fn(String) -> String + Send>>,
}

impl Callbacks {
    fn register<F>(&mut self, f: F) -> u32
    where
        F: 'static + Send + Fn(String) -> String,
    {
        let id = self.current;
        self.current += 1;

        self.registered.insert(id, Box::new(f));

        id
    }

    fn call(&self, id: u32, args: String) -> String {
        let f = self.registered
            .get(&id)
            .expect(
                "FATAL: Failed to find callback. Make sure to register all\
                 callbacks."
            );

        f(args)
    }
}

pub fn register<F, A, R>(f: F) -> u32
where
    F: 'static + Send + Fn(A) -> R,
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
    let callbacks = HANDLER.lock().unwrap();

    // Optimize for the null pointer.
    match callbacks.call(id, args) {
        ref s if s == "null" => None,
        s => Some(s),
    }
}
