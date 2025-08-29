use std::{self, fmt::Formatter};
use discard::Discard;




#[derive(Debug)]
pub(crate) struct Callbacks {
    pub(crate) after_insert: Vec<InsertCallback>,
    pub(crate) after_remove: Vec<RemoveCallback>,
    trigger_remove: bool
}

impl Callbacks {
    pub(crate) fn new() -> Self {
        Self {
            after_insert: vec![],
            after_remove: vec![],
            trigger_remove: true,
        }
    }

    pub(crate) fn after_insert<A: FnOnce(&mut Callbacks) + 'static>(&mut self, callback: A) {
        // self.after_insert.push(InsertCallback(Box::new(callback)));
    }

    pub(crate) fn after_remove<A: Discard + 'static>(&mut self, value: A) {
        // self.after_remove.push(RemoveCallback(Box::new(value)));
    }

    pub(crate) fn trigger_after_insert(&mut self) {
        if !self.after_insert.is_empty() {
            let mut callbacks = Callbacks::new();
            std::mem::swap(&mut callbacks.after_remove, &mut self.after_remove);
            for f in self.after_insert.drain(..) {
                f.0(&mut callbacks);
            }
            self.after_insert = vec![];
            assert_eq!(callbacks.after_insert.len(), 0);
            std::mem::swap(&mut callbacks.after_remove, &mut self.after_remove);
        }
    }

    pub(crate) fn trigger_after_remove(&mut self) {
        for f in self.after_remove.drain(..) {
            f.0.remove();
        }
    }

    pub(crate) fn leak(&mut self) {
        self.trigger_remove = false;
    }

}

trait IRemove {
    fn remove(self: Box<Self>);
}
impl<A: Discard> IRemove for A {
    fn remove(self: Box<Self>) {
        self.discard();
    }
}

#[repr(transparent)]
pub(crate) struct InsertCallback(Box<dyn FnOnce(&mut Callbacks)>);

#[repr(transparent)]
pub(crate) struct RemoveCallback(Box<dyn IRemove>);

impl std::fmt::Debug for InsertCallback {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "InsertCallback")
    }
}

impl std::fmt::Debug for RemoveCallback {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "RemoveCallback")
    }
}