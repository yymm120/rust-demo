#[allow(unused)]


#[cfg(test)]
mod lock_test {
    use std::{sync::{Mutex, Arc, RwLock}, thread};

    #[test]
    fn mutex_basic() {
        let m = Mutex::new(5);
        {
            // note: MutexGuard<T>是一个智能指针, 实现Deref和Drop特征. 自动解引用后获得一个指向内部数据的引用.
            let mut num = m.lock().unwrap();
            *num = 6;
        }
        println!("m = {:?}", m);
    }

    // 10个线程, 每个线程都会执行`counter += 1`. 预期结果是 `counter == 10;`
    #[test]
    fn mutex_in_multi_thread() {
        let counter = Arc::new(Mutex::new(0));
        let mut handles = vec![];
        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                let mut num = counter.lock().unwrap();
                *num += 1;
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        println!("Result is {}", *counter.lock().unwrap());
    }


    // try_lock 会尝试获取一次锁, 如果无法获取会立刻返回一个错误.
    #[test]
    fn mutex_method_try_lock() {
        ()
    }

    #[test]
    fn read_write_lock() {
        let lock = RwLock::new(5);
        // 允许多个读
        {
            let r1 = lock.read().unwrap();
            let r2 = lock.read().unwrap();
            assert_eq!(*r1, 5);
            assert_eq!(*r2, 5);
        }// auto drop

        // 只允许一个写
        {
            let mut w = lock.write().unwrap();
            *w += 1;
            assert_eq!(*w, 6);
        } // auto drop
    }
}


#[cfg(test)]
mod condavar_test {
    use std::{sync::{Arc, Condvar, Mutex}, thread};
    use std::time::Duration;
    
    /// 在`thread_condition` 测试函数里实现了condvar条件控制线程在状态满足时执行
    /// 这里利用`condvar`控制线程多线程同步执行
    /// 预期输出： 
    ///     outside counter: 1
    ///     inner counter: 1
    ///     outside counter: 2
    ///     inner counter: 2
    ///     outside counter: 3
    ///     inner counter: 3
    #[test]
    fn condvar_control_thread() {
        let flag = Arc::new(Mutex::new(false));
        let cond = Arc::new(Condvar::new());
        let flag_clone = flag.clone();
        let cond_clone = cond.clone();

        let handle = thread::spawn(move || {
            let mut lock = flag_clone.lock().unwrap();
            let mut counter = 0;
            while counter < 3 {
                while !*lock {
                    lock = cond_clone.wait(lock).unwrap();
                }
                *lock = false;
                counter += 1;
                println!("inner counter: {}", counter);
            }
        });

        let mut counter = 0;
        loop {
            thread::sleep(Duration::from_millis(1000));
            *flag.lock().unwrap() = true;
            counter += 1;
            if counter > 3 {
                break;
            }
            println!("outside counter: {}", counter);
            cond.notify_one();
        }
        handle.join().unwrap();
        println!("{:?}", flag);
    }
}

/// 信号量
#[cfg(test)]
mod semaphore {
    use std::sync::Arc;
    // use tokio::sync::Semaphore;

    #[test]
        fn semaphore_control_concurrente_number() {
            // let semaphore = Arc::new(Semaphore::new(3));
            // let mut join_handles = Vec::new();
            // for _ in 0..5 {
            //     let permit = semaphore.clone().acquire_owned().await.unwrap();
            //     join_handles.push(tokio::spawn(async move {
            //         // do somethin
            //         drop(permit);
            //     }));
            // }
            // for handle in join_handles {
            //     handle.await.unwrap();
            // }
            ()
        }
}

fn main () {
}