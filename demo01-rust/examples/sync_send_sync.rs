#![allow(unused)]


/// `Rc`, `RefCell`和`裸指针`不能在多线程使用, 因为没有标记`Send`和`Sync`特征. (该特征未定义任何行为, 仅仅用作标记)
/// 
/// Send和Sync的作用:
///   - `Send` trait 可以在线程间安全的传递所有权. 所以Arc<T>中的T一定要实现`Send`. (索性大部分rust类型都实现了`Send`和`Sync`, 注意: 若是复合类型若成员都实现了它们, 则它自动实现`Send`和`Sync`.)
///   - `Sync` trait 可以在线程间安全的共享(通过引用). 换句话说, 并发读的安全由`Sync`特征保证.
/// 
/// ```rust
/// // Rc源码片段
/// impl<T: ?Sized> !marker::Send for Rc<T> {}
/// impl<T: ?Sized> !marker::Sync for Rc<T> {}
/// 
/// // Arc源码片段
/// unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
/// unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}
/// 
/// // RwLock源码片段
/// unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}
/// ```
/// 
/// rust 大多数类型都实现了`Send`和`Sync`, 但有些例外:
/// 1. 裸指针两者都没实现.
/// 2. UnsafeCell为实现`Sync`, `Cell`和`RefCell`也没实现`Sync`
/// 3. Rc两者都没实现.
/// 
/// 手动实现`Send`和`Sync`是不安全的, 需要使用unsafe自主提供并发安全保证.
/// 
/// 
/// 测试函数:
/// `implement_send_for_raw_pointer`        为裸指针实现`Send`
/// `implement_sync_for_raw_pointer`        为裸指针实现`Sync`
/// 

#[cfg(test)]
mod send_test {
    use std::borrow::BorrowMut;
    use std::thread;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;


    /// 为裸指针实现`send`
    #[test]
    fn implement_send_for_raw_pointer() {

        #[derive(Debug)]
        struct MyBox(*mut u8);          // 1. 必须使用`newtype`
        unsafe impl Send for MyBox {}   // 2. 必须实现`Send`, 才能move传递所有权

        let p = MyBox(5 as *mut u8);
        let t = thread::spawn(move || {
            println!("{:?}", p);
        });
        t.join().unwrap();
    }


    /// 为裸指针实现`sync`
    #[test]
    fn implement_sync_for_raw_pointer() {
        #[derive(Debug)]
        struct MyBox(*const u8);        // 1. 必须使用`newtype`
        unsafe impl Send for MyBox {}   // 2. 必须实现Send, 才能move进子线程, 传递所有权.
        unsafe impl Sync for MyBox {}   // 3. 必须实现Sync, 才能安全的使用`clone`和`&`, 实现数据共享

        let p = &MyBox(5 as *mut u8);
        let p_arc = Arc::new(Mutex::new(p));    // 多线程共享数据需要`Arc`和`Mutex`配合.
        let p_arc_clone = p_arc.clone();

        let thread = thread::spawn(move || {
            let mut v1 = p_arc.lock().unwrap();
            *v1 = &MyBox(9 as *mut u8);
            println!("{:?}", v1.0);
        });
        thread::sleep(Duration::from_millis(1));
        println!("{:?}", p_arc_clone.lock().unwrap().0);
        thread.join().unwrap();
    }

}

fn main() {} 