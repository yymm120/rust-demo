#![allow(unused)]
use std::hint;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

///
/// 测试函数: 
/// `atomic_ordering`   用Ordering确定代码执行顺序. (在使用Atomic时, 执行顺序不一定等于代码编写的顺序)
/// `atomic_bool`       AtomicBool类型的使用.
/// `atomic_and_arc`    Arc实现Atomic数据共享.




/// cargo watch -q -c -x "test --example future atomic -- --nocapture"
#[test]
fn atomic_bool() {
    // note: Atomic作为全局变量来使用, 即使多个线程对它修改, 它也不会出现脏数据
    static STATE: AtomicBool = AtomicBool::new(false);
    let thread1 = thread::spawn(move || {
        for _ in 0..10 {
            STATE.fetch_not(Ordering::Relaxed);
        }
    });
    thread1.join().unwrap();
    println!("state is {:?}", STATE);
}

#[test]
fn atomic_ordering() {
    // Relaxed: 宽松, 乱序
    // Release: 释放, 设定内存屏障barrier, 保障它之前的操作永远在它之前. 它之后的操作可能在它之前
    // Aqcuire: 获取, 设定内存屏障, 保证它之后的访问永远在它之后, 但是它之前的操作有可能在它之后.
    // AcqRel: Acquire和Relase的结合, 保证它之前的永远在它之前, 它之后的也永远在它之后.
    // SeqCst: 顺序一致性

    static mut DATA: u64 = 0;
    static READY: AtomicBool = AtomicBool::new(false);
    fn reset () {
        unsafe {
            DATA = 0;
        }
        READY.store(false, Ordering::Relaxed);
    }
    fn producer() -> JoinHandle<()> {
        thread::spawn(move || {
            unsafe {
                DATA = 100;                                    // A 一定在下面代码之前先执行
            }
            READY.store(true, Ordering::Release);   // B: 内存屏障 ↑
        })
    }
    fn consumer() -> JoinHandle<()> {
        thread::spawn(move || {
            while !READY.load(Ordering::Acquire) {}      // C: 内存屏障 ↓
            assert_eq!(100, unsafe { DATA });                   // D 一定在上面代码之后执行
        })
    }

    loop {
        reset();    // in main thread
        let t_producer = producer(); // in producer thread
        let t_consumer = consumer(); // in consumer thread
        t_consumer.join().unwrap();
        t_producer.join().unwrap();
        break;
    }
}


#[test]
fn atomic_and_arc() {
    let spinlock = Arc::new(AtomicUsize::new(1));
    let spinlock_clone = Arc::clone(&spinlock);
    let thread = thread::spawn(move || {
        spinlock_clone.store(0, Ordering::SeqCst); // 等效于释放锁
    });
    while spinlock.load(Ordering::SeqCst) != 0 {    // 检查锁
        hint::spin_loop();
    }
    if let Err(panic) = thread.join() {
        println!("Thread had an error: {:?}", panic);
    }
}

/// atomic仅仅包含数值类型的原子操作.
/// 某些情况必须使用锁, 例如Mutex配合Condvar控制线程

/// Atomic 可用来实现:
///     1. 无锁数据结构
///     2. 全局变量, 例如全局自增id
///     3. 跨线程计数器, 例如统计指标


fn main() {

}