use std::future::Future;

use discard::DiscardOnDrop;
use futures_signals::{cancelable_future, signal::{Signal, SignalExt}, CancelableFutureHandle};
use futures_util::future::ready;
use wasm_bindgen_futures::spawn_local;


pub(crate) fn spawn_future<F>(future: F) -> DiscardOnDrop<CancelableFutureHandle>
    where F: Future<Output = ()> + 'static
{
    let (handle, future) = cancelable_future(future, || ());
    spawn_local(future);
    handle
}

pub(crate) fn for_each<A, B>(signal: A, mut callbacks: B) -> CancelableFutureHandle
    where A: Signal + 'static, B: FnMut(A::Item) + 'static
{
    DiscardOnDrop::leak(spawn_future(signal.for_each(move |value| {
        callbacks(value);
        ready(())
    })))
}