use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;

fn main() {
    // let running = Arc::new(AtomicBool::new(true));
    // let r = running.clone();
    // set_handler(move || {
    //     r.store(false, SeqCst);
    // });
    // 
    // while running.load(SeqCst) {
    //     println!("working...");
    //     // TODO
    // }
    // println!("exiting...");

}