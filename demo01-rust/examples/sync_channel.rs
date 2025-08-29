#![allow(unused)]
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// chanel

/// `channel_mpsc_one_send_one_receive` 单个发送者, 单个接收者

/// std::sync::mpsc 中`mpsc`是`multiple producer, single consumer`的缩写.
///
/// mpsc支持多个发送者, 但只支持单个接收者
///
///
/// 运行: `cargo watch -q -c -x "test --example channel channel_mpsc -- --nocapture"`
#[test]
fn channel_mpsc_one_send_one_receive() {
    // 创建发送者tx, 接收者rx
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        // 发送
        tx.send(1).unwrap();
    });
    // 接收
    println!("receive {}", rx.recv().unwrap()); // note: rx.recv()会阻塞当前线程, 直到取得值或者通道被关闭.
}

#[test]
fn channel_mpsc_try_recv() {
    // 创建发送者tx, 接收者rx
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        // 发送
        tx.send(1).unwrap();
    });
    // 接收
    thread::sleep(Duration::from_millis(1));
    println!("receive {}", rx.try_recv().unwrap()); // note: rx.try_recv()不会阻塞当前线程, 如果没有立即取得消息, 会立刻返回一个错误.
}

#[test]
fn channel_mpsc_ownership() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let a = String::from("Hello");
        tx.send(a).unwrap(); // note: 字符串String底层存储在堆上, 没有实现Copy特征, 发送时会将所有权转移给receive
    });
    println!("receive {}", rx.recv().unwrap());
}

#[test]
fn channel_mpsc_for_receive() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("thread"),
        ];
        for val in vals {
            tx.send(val).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });
    for received in rx {
        // note: 主线程用for循环阻塞从子循环中接收数据.
        println!("Got {}", received);
    }
}

/// 由于子线程会拿走发送者的所有权, 所以必须对发送者进行克隆. 让每个线程拿走一份拷贝
#[test]
fn channel_mpsc_multi_sender() {
    let (tx_origin, rx) = mpsc::channel();
    let tx_clone = tx_origin.clone();
    thread::spawn(move || {
        tx_origin.send(String::from("hi from origin tx")).unwrap();
    });
    thread::spawn(move || {
        tx_clone.send(String::from("hi from cloned tx")).unwrap();
    });
    for received in rx {
        // note: 需要所有发送者都被drop掉, rx才会收到错误, 才能跳出for循环.
        println!("Got {}", received);
    }
}

/// 消息是有顺序的, 但是线程的执行是无序的.

/// 同步和异步channel

/// 若是不做任何处理, 通道都是异步的.
/// 无论接收者是否正在接收消息, 消息发送者在发送消息时不会阻塞.
#[test]
fn channel_mpsc_async() {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        println!("发送之前");
        tx.send(1).unwrap();
        println!("发送之后");
    });

    println!("睡眠之前");
    thread::sleep(Duration::from_secs(3));
    println!("睡眠之后");

    println!("receive {}", rx.recv().unwrap());
    handle.join().unwrap();
}

fn sync_channel(num: usize) {
    // 设置消息缓存数量
    let (tx, rx) = mpsc::sync_channel(num);
    let handle = thread::spawn(move || {
        println!("发送之前");
        tx.send(1).unwrap(); // note: sync_channel缓存数设置为0时会在这里阻塞, 直到消息被接收.
        println!("发送之后");
        tx.send(1).unwrap(); // note: sync_channel缓存数设置为1时会在这里阻塞, 直到消息被接收.
        println!("又发送了");
    });
    println!("睡眠之前");
    thread::sleep(Duration::from_secs(3));
    println!("睡眠之后");

    println!("receive {}", rx.recv().unwrap());
    handle.join().unwrap();
}

#[test]
fn channel_mpsc_sync_0() {
    sync_channel(0);
}

#[test]
fn channel_mpsc_sync_1() {
    sync_channel(1);
}

#[test]
fn channel_mpsc_close_channel() {
    // 所有发送者被drop或者所有接收者被drop后, 通道自动关闭.
    // 并且这是在编译期完成的.
    ()
}

///
#[test]
fn channel_mpsc_other_data_struct() {
    // Rust 会按照枚举中占用内存最大的那个成员进行内存对齐，这意味着就算你传输的是枚举中占用内存最小的成员，它占用的内存依然和最大的成员相同, 因此会造成内存上的浪费
    enum Fruit {
        Apple(u8),
        Orange(String),
    }
    let (tx, rx): (Sender<Fruit>, Receiver<Fruit>) = mpsc::channel();
    tx.send(Fruit::Orange("sweet".to_string())).unwrap();
    tx.send(Fruit::Apple(2)).unwrap();

    for _ in 0..2 {
        match rx.recv().unwrap() {
            Fruit::Apple(count) => println!("received {} apples", count),
            Fruit::Orange(flavor) => println!("received {} oranges", flavor),
        }
    }
}

#[test]
fn channel_mpsc_manual_drop() {
    let (send, recv) = mpsc::channel();
    let num_threads = 3;
    for i in 0..num_threads {
        let thread_send = send.clone(); // clone
        thread::spawn(move || {
            thread_send.send(i).unwrap();
            println!("thread {:?} finished", i);
        });
    }

    drop(send); // note: 手动drop, 因为前面只使用了clone. 原send变量还没有被drop.

    for x in recv {
        println!("Got {}", x);
    }
}

#[test]
fn channel_mpmc_tools() {
    // 如果你需要 mpmc(多发送者，多接收者)或者需要更高的性能，可以考虑第三方库:

    // crossbeam-channel, 老牌强库，功能较全，性能较强，之前是独立的库，但是后面合并到了crossbeam主仓库中
    // flume, 官方给出的性能数据某些场景要比 crossbeam 更好些
    ()
}

fn main() {}
