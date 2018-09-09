//! Support for `futures 0.1.x`.

use std::{
    collections::HashMap,
    sync::Arc,
};

use futures_v01x::{
    executor::{self, Notify, Spawn},
    Async,
    Future,
};
use global::Global;

type Boxed = Box<Future<Item = (), Error = ()> + 'static + Send>;

#[derive(Default)]
struct Pool {
    counter: usize,
    futures: HashMap<usize, Spawn<Boxed>>,
}

impl Pool {
    fn next(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
}

static POOL: Global<Pool> = Global::INIT;

fn poll(id: usize) {
    let mut spawn = match POOL.lock().futures.remove(&id) {
        Some(s) => s,
        None => return,
    };

    let poll = spawn.poll_future_notify(&Arc::new(StasisNotify), id);

    match poll {
        Ok(Async::NotReady) => {
            POOL.lock()
                .futures
                .insert(id, spawn);
        }

        Ok(Async::Ready(())) => (),
        Err(()) => (),
    }
}

#[derive(Clone)]
struct StasisNotify;

impl Notify for StasisNotify {
    fn notify(&self, id: usize) {
        poll(id);
    }

    fn drop_id(&self, id: usize) {
        let _ = POOL
            .lock()
            .futures
            .remove(&id);
    }
}

/// Spawn a future.
pub fn spawn<F: 'static + Send + Future<Item = (), Error = ()>>(f: F) {
    let spawn = executor::spawn(Box::new(f) as Boxed);

    let mut guard = POOL.lock();
    let id = guard.next();
    guard.futures.insert(id, spawn);

    drop(guard);

    poll(id);
}
