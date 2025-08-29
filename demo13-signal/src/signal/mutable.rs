use std::{sync::{atomic::{AtomicBool, Ordering}, Mutex}, task::Waker};




pub(crate) struct ChangedWaker {
    changed: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

impl ChangedWaker {
    pub(crate) fn new() -> Self {
        Self {
            changed: AtomicBool::new(true),
            waker: Mutex::new(None),
        }
    }

    pub(crate) fn wake(&self, changed: bool) {
        let waker = {
            let mut lock = self.waker.lock().unwrap();
            if changed {
                self.changed.store(true, Ordering::SeqCst);
            }
            lock.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        } 
    }
}