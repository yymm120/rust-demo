#![allow(unused)]
use std::{
    cell::RefCell,
    sync::{
        self,
        atomic::{AtomicBool, Ordering},
        Arc, Barrier, Condvar, Mutex, Once
    },
    thread::{self, sleep},
    time::Duration,
};
use std::sync::OnceLock;
use std::time::Instant;

/// 运行: `cargo watch -q -c -x "test --example thread thread -- --nocapture`
///
/// 参考文档: https://course.rs/advance/concurrency-with-threads/thread.html
///
/// 总结:
///     1. spawn():          创建线程使用`thread::spawn`函数
///     2. join().unwrap():  主线程等待子线程执行完毕 `thread.join().unwrap;`
///     3. spawn(move ||{}): 必须使用`move`将所有权转移给闭包.
///     4. don't dead loop:  rust不提供杀死线程的接口, 需要有意识的防止死循环跑满CPU.
///     5. 0.24ms:           约0.24ms创建一个线程
///     6. cas:              cas虽然无锁, 但线程数量增多时, cas重试次数会增加, 从而影响性能.
///     7. barrier:          线程屏障让多个线程都执行到某个点后, 再一起往后执行.
///     8. thread_local!     线程局部变量, 每个线程独享, 通过`with`方法访问.
///     9. condvar           条件变量可以挂起线程`.wait(guard)`, 直到满足条件时通知线程继续执行`.notify_one()`
///     10. Once cell_once   多线程环境下只会被初始化一次. (其他线程会忽略). 如果每次初始化的值不一样, 会导致多次执行结果不一致.
///
#[test]
fn thread_spawn() {
    let thread = thread::spawn(|| {
        for i in 1..10 {
            println!("hi, thread number is {}", i);
            thread::sleep(Duration::from_millis(1));
        }
    });
    // note: 如果注释掉以下行, for循环可能只输出一次.
    thread.join().unwrap(); // 等待子线程执行结束
}

#[test]
fn thread_spawn_move() {
    let v = vec![1, 2, 3];
    // note: 如果变量`v`没有所有权, 那么编译会报错, 因此`move`是必须的
    let thread = thread::spawn(move || {
        println!("{:?}", v);
    });

    thread.join().unwrap();
}


/// CAS 无锁同步
///
/// 虽然是无锁的, 但线程增多时, 大量线程同时访问会让CAS重试次数增加. 从而影响性能.
#[test]
fn thread_cas() {
    /// 这是一个无锁的CAS实现. 暂时未完成.
    /// ```rust
    /// for i in 0..5 {
    ///     let ht = Arc::clone(&ht);
    ///     let handle = thread::spawn(move || {
    ///         for j in 0..adds_per_thread {
    ///             let key = thread_rng().gen::<u32>();
    ///             let value = thread_rng().gen::<u32>();
    ///             ht.set_item(key, value);
    ///         }
    ///     });
    ///     handle.push(handle);
    /// }
    /// for handle in handles {
    ///     handle.join().unwrap();
    /// }
    /// ```

    // 已完成的无锁等待
    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = flag.clone();

    // 线程 B 等待线程 A
    let thread_b = thread::spawn(move || {
        // 自旋等待，直到线程 A 完成
        while !flag.load(Ordering::Acquire) {
            // 加一个小的休眠避免忙等消耗太多 CPU
            thread::yield_now(); // 线程让出时间片, 但是不会休眠
            std::hint::spin_loop(); // 告诉CPU这是自旋循环
        }
        println!("线程 B 执行");
    });

    // 线程 A 先执行
    let thread_a = thread::spawn(move || {
        println!("线程 A 执行");
        flag_clone.store(true, Ordering::Release); // 设置完成标志
    });

    thread_a.join().unwrap();
    thread_b.join().unwrap();
    ();
}

/// 线程屏障让多个线程都执行到某个点后, 再一起往后执行.
///
/// 屏障会使线程休眠, 直到被唤醒.
///
/// output:
///     before wait
///     before wait
///     before wait
///     after wait
///     after wait
///     after wait
#[test]
fn thread_barrier() {
    let mut handles = Vec::with_capacity(3);
    let barrier = Arc::new(Barrier::new(3));
    for _ in 0..3 {
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            println!("before wait");
            b.wait();
            println!("after wait");
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}


/// thread_local! 为每个线程分配一个同名变量, 并非共享
///
/// cargo watch -q -c -x "test --example thread thread_local -- --nocapture
#[test]
fn thread_local() {
    thread_local! {
        static FOO: RefCell<u32> = RefCell::new(1)
    };
    let thread1 = thread::spawn(move || {
        FOO.with(|f| {
            assert_eq!(*f.borrow(), 1);
            *f.borrow_mut() = 2;
        });
    });
    let thread2 = thread::spawn(move || {
        FOO.with(|f| {
            assert_eq!(*f.borrow(), 1);
            *f.borrow_mut() = 3;
        });
    });
    thread1.join().unwrap();
    thread2.join().unwrap();
    // 无论子线程如何修改, 都不会影响主线程, 因为`thread_local!`会为每个线程分配一个FOO变量
    FOO.with(|f| {
        assert_eq!(*f.borrow(), 1); 
    })
}



/// Condvar 条件变量控制线程的挂起和执行.
/// 
/// 代码解释： 挂起主线程, 直到condvar的值发生改变再继续执行.
/// 
/// 运行: `cargo watch -q -c -x "test --example thread thread_condition -- --nocapture`
#[test]
fn thread_condition() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();

    let thread1 = thread::spawn(move || {
        // step3: 获取lock和condvar, 执行相关代码.
        println!("2. into child thread.");
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        println!("3. changing state");
        // step4: 修改状态, 并通知主线程继续运行.
        *started = true;
        cvar.notify_one();
    });

    
    // step1: 首先执行以下代码, 
    println!("1. start");
    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        // step2: 以下代码挂起主线程, 并等待子线程运行. (cvar.wait方法会释放锁.)
        started = cvar.wait(started).unwrap();
        // step5: 主线程继续执行, 并跳出while循环.
        println!("4. reback main thread.");
    }
    println!("5. end");
}




/// # Once: 两个线程, 只有其中一个能执行初始化.
///
/// 运行: cargo watch -q -c -x "test --example thread thread_once -- --nocapture"
#[test]
fn thread_once() {
    static mut VAL: usize = 0;  // 初始为0
    static INIT: Once = Once::new();

    let thread1 = thread::spawn(move || {
        thread::sleep(Duration::from_millis(1));
        INIT.call_once(|| {
            unsafe { VAL = 1; }; // 设置为1
        });
    });
    let thread2 = thread::spawn(move || {
        // thread::sleep(Duration::from_millis(2));
        INIT.call_once(|| {
            unsafe { VAL = 2; }; // 设置为2
        });
    });

    thread1.join().unwrap();
    thread2.join().unwrap();

    println!("{}", unsafe { VAL }); // 输出可能是1, 可能是2
}



fn main() {}


/// 一种优化CAS自旋忙等待的方式
fn optimized_spin_wait(flag: &AtomicBool) {
    let mut short_spin = 0;
    let start = Instant::now();

    while !flag.load(Ordering::Acquire) {
        short_spin += 1;

        // 第一阶段：快速自旋（1000次）
        if short_spin < 1000 {
            std::hint::spin_loop();
            continue;
        }

        // 第二阶段：偶尔让出CPU
        if short_spin % 100 == 0 {
            thread::yield_now();
        }

        // 第三阶段：长时间等待后休眠
        if start.elapsed() > Duration::from_millis(1) {
            thread::sleep(Duration::from_micros(10));
        }
    }
}
