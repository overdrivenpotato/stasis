//! Support for `futures 0.2.x`.

use std::{
    collections::HashMap,
    sync::Arc,
};

use global::Global;
use futures_v02x::{
    executor::{Executor, SpawnError},
    task::{Context, LocalMap, Waker, Wake},
    Future,
    Never,
    Async,
};

static POOL: Global<Pool> = Global::INIT;

type Boxed = Box<Future<Item = (), Error = Never> + 'static + Send>;

#[derive(Default)]
struct Pool {
    current: u32,
    futures: HashMap<u32, Boxed>,
}

struct StasisWake {
    id: u32,
}

impl Wake for StasisWake {
    fn wake(arc_self: &Arc<Self>) {
        StasisExecutor.poll(arc_self.id);
    }
}

/// An executor.
///
/// This can be freely constructed without any function calls.
pub struct StasisExecutor;

impl StasisExecutor {
    fn poll(&mut self, id: u32) {
        let mut f = match POOL.lock().futures.remove(&id) {
            Some(f) => f,
            None => return,
        };

        let poll = {
            let mut map = LocalMap::new();
            let waker = Waker::from(Arc::new(StasisWake { id }));
            let mut context = Context::new(&mut map, &waker, self);

            f.poll(&mut context)
        };

        match poll {
            // Re-insert if pending.
            Ok(Async::Pending) => {
                POOL.lock()
                    .futures
                    .insert(id, f);
            }

            Ok(Async::Ready(())) => (),
            Err(e) => e.never_into(),
        }
    }
}

impl Executor for StasisExecutor {
    fn spawn(&mut self, f: Boxed) -> Result<(), SpawnError> {
        let mut lock = POOL.lock();

        let id = lock.current;
        lock.current += 1;

        lock.futures.insert(id, f);

        // Important: this must be dropped before poll to avoid deadlock.
        drop(lock);

        self.poll(id);

        Ok(())
    }
}

/// Spawn a future.
pub fn spawn<F: 'static + Send + Future<Item = (), Error = Never>>(f: F) {
    StasisExecutor
        .spawn(Box::new(f))
        .expect("StasisExecutor failed to spawn. This should never happen.");
}
