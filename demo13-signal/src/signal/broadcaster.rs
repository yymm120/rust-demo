#![allow(unused)]
use std::sync::{atomic::{AtomicBool, Ordering}, Mutex, Weak};

use super::mutable::ChangedWaker;



#[derive(Debug)]
struct BroadcasterNotifier {
    is_changed: AtomicBool,
    waker: Mutex<Vec<Weak<ChangedWaker>>>,
}

impl BroadcasterNotifier {
    fn new() -> Self {
        Self {
            is_changed: AtomicBool::new(true),
            waker: Mutex::new(vec![]),
        }
    }

    fn notify(&self) {
        let mut lock = self.waker.lock().unwrap();
        self.is_changed.store(true, Ordering::SeqCst);
        lock.retain(|waker| {
            if let Some(waker) = waker.upgrade() {
                waker.wake(false);
                true
            } else {
                false
            }
        })
    }

    fn is_changed(&self) -> bool {
        self.is_changed.swap(false, Ordering::SeqCst)
    }
}

#[test]
fn dev_test_broadcaster() {
    let broadcaster_notifier = BroadcasterNotifier::new();
    println!("{:?}", broadcaster_notifier);
    println!("{:?}", broadcaster_notifier.is_changed);
    broadcaster_notifier.is_changed.store(false, Ordering::Acquire);
    broadcaster_notifier.notify();
}