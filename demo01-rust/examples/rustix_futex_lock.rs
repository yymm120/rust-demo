#![allow(unused)]
use std::{env, fs, ptr, thread};
use std::fs::OpenOptions;
use std::os::fd::AsFd;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use rustix::mm::{mmap, MapFlags, ProtFlags};
use rustix::thread::futex;

/// 使用 rustix_futex_sync 中的同步工具
///
/// 这里面的同步工具, 可以通过监听共享内存中的原子类型数据, 完成对多进程中的线程强制休眠等待和批量唤醒功能.
///
/// 它获取锁的源码实现很简单, 做100次spin自旋, 失败就休眠. 借助共享内存 CAS 操作 + futex休眠唤醒功能来实现的.
///
/// 事实上除了 futex 可以唤醒, event也可以, 但是 futex 的应该性能会好一些.
///
/// Futex 优于 Event 优于 Signal, (这么排序也许有错, 因为目前的知识水平有限.)
///

use rustix_futex_sync::shm::Mutex;
use rand::{random, random_range};

#[repr(C)]  // 字段顺序按照声明顺序排列（Rust默认可能重排）
#[repr(align(64))]  //  强制类型按指定字节数对齐 64 或者 64的倍数
#[derive(Debug )]
struct SharedData {
    count: AtomicI32,   // 4个字节
    count_a: i32,   // 4个字节
    _padding: [u8; 56], // 手动填充到64个字节
}

/// 这个程序说明:  rustix_futex_sync 中的 Mutex 没有锁住
///
/// 从源码来看, Mutex在内部使用的  Atomic::new, 这没办法做多进程下工作
///
/// 也许可以自行实现一个从 共享内存中映射出来的一个锁.
///
fn main() -> anyhow::Result<()>{
    let op = env::args().collect::<Vec<_>>().get(1).unwrap_or(&"both".to_string()).clone();
    if fs::exists("rustix_futex_lock.bin")? {
        fs::remove_file("rustix_futex_lock.bin")?;
    }
    println!("{}", op);


    if op == "mutex" {
        // 创建共享内存
        let file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("rustix_futex_lock.bin")?;
        let size = std::mem::size_of::<SharedData>();
        file.set_len(size as u64);
        multi_processor_with_mutex();
    } else if op == "rw" {
        while !std::fs::exists("rustix_futex_lock.bin")? {
            std::hint::spin_loop();
            thread::sleep(Duration::from_secs(1));
            println!("not exists");
        }
        multi_processor_with_wr();
    }

    Ok(())
}


/// Mutex的用法和标准库的用法基本相同
fn multi_processor_with_mutex() -> anyhow::Result<()>{
    let file = OpenOptions::new().read(true).write(true).open("rustix_futex_lock.bin")?;
    let size = std::mem::size_of::<SharedData>();
    let borrowed_fd= file.as_fd();

    // // 创建共享内存映射 - 指针
    let shared_ptr = unsafe {
        mmap(
            ptr::null_mut(),                        // 自动分配地址
            size,                                   // 共享内存的大小
            ProtFlags::READ | ProtFlags::WRITE, // 读和写对其他进程可见
            MapFlags::SHARED,                       // shared
            borrowed_fd,                            // 文件描述符
            0,                                 // 偏移量
        )? as *mut SharedData
    };


    let data = unsafe {&mut *shared_ptr};

    let data = Arc::new(Mutex::new(data));

    let data_clone = data.clone();
    let data_clone2 = data.clone();

    let op = env::args().collect::<Vec<_>>().get(2).unwrap_or(&"both".to_string()).clone();

    if op == "lock" {
        thread::spawn(move || {
            loop {
                println!("lock: {:?}", data_clone.lock().count_a);
                thread::sleep(Duration::from_secs(2));
            }
        });

        let a = data.lock();
        println!("{:?}", a);
        println!("is locked: {}", data.is_locked());
        thread::sleep(Duration::from_secs(100));
    }


    if op == "read" {

        thread::spawn(move || {
            loop {
                println!("lock: {:?}", data_clone2.lock().count_a);
                thread::sleep(Duration::from_secs(2));
            }
        });
        {
            thread::sleep(Duration::from_secs(5));
            let mut d = data.lock();
            println!("is locked: {}", data.is_locked());
            let x = rand::random::<u8>();
            d.count_a = x as i32;
        }

    }

    thread::sleep(Duration::from_secs(100));
    Ok(())
}

fn multi_processor_with_wr() {

}









