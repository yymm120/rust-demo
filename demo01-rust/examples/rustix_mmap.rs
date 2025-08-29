#![allow(unused)]
use std::{env, ptr, thread};
use std::fs::{File, OpenOptions};
use std::ops::Deref;
use std::os::fd::AsFd;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{fence, AtomicI32, AtomicI64, AtomicU64, Ordering};
use std::time::Duration;
use rustix::mm::{mmap, munmap, MapFlags, ProtFlags};
use rustix_futex_sync::shm::Mutex;

#[repr(C)]  // 字段顺序按照声明顺序排列（Rust默认可能重排）
#[repr(align(64))]  //  强制类型按指定字节数对齐 64 或者 64的倍数
#[derive(Debug )]
struct SharedData {
    count: AtomicI32,   // 4个字节
    count_a: i32,   // 4个字节
    _padding: [u8; 56], // 手动填充到64个字节
}

unsafe impl Sync for SharedData {}
unsafe impl Send for SharedData {}


/// memmap2 有一个问题， 对共享内存修改一亿次后, 两边的数据不同.
///
/// 数据不一致使得memmap2的稳定性大大降低, 在程序中是致命的.
///
/// 考虑使用rustix的mmap, 目前还没有看到mmap有数据不一致的问题. 至少在一亿次修改之后, 没有出现数据不一致的问题.
///
/// 用以下命令启用两个进程, 同时原子加200000000次和减200000000此, 可以看到数据最后能够归零.
///
/// 这个案例说明: 跨进程对共享内存可以进行原子操作.
///
/// ```
/// cargo watch -w examples -x "run --example rustix_mmap -- sub"
/// cargo watch -w examples -x "run --example rustix_mmap -- add"
/// ```
fn main() -> anyhow::Result<()>{
    if std::fs::exists("rustix_mmap.bin")? {
        std::fs::remove_file("rustix_mmap.bin")?;
    }
    thread::sleep(Duration::from_secs(2)); // 等一会
    let op = env::args().collect::<Vec<_>>().get(1).unwrap_or(&"both".to_string()).clone();
    let size = std::mem::size_of::<SharedData>();
    let file;
    if op == "add" {
        file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("rustix_mmap.bin")?;
        file.set_len(size as u64);
    } else {
        while !std::fs::exists("rustix_mmap.bin")? {
            std::hint::spin_loop();
            println!("not exists");
        }
        file = OpenOptions::new().read(true).write(true).open("rustix_mmap.bin")?;
    }

    let borrowed_fd= file.as_fd();
    println!("{:?}", borrowed_fd);
    println!("{:?}", op);

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
    println!("mem ptr: {:?}", shared_ptr);

    let a = unsafe {&mut *shared_ptr};
    let p = &a as *const _;

    println!("data ptr: {:?}", &a as *const _);

    /// 在多进程下, 原子类型的地址是不同的, 因此Atomic操作无法在多进程下生效
    // let b = unsafe { (a.count).as_ptr() };


    /// rustix_futex_sync
    // let lock = Mutex::new(a);
    for i in 0..200000000 {
        {
            // let mut data = lock.lock();
            if op == "sub" {
                a.count_a -= 1;
                // a.count.fetch_sub(1, Ordering::SeqCst);
            } else {
                a.count_a += 1;
                // a.count.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    loop {
        let a1 = unsafe {&mut *shared_ptr};
        // let a = unsafe {&mut *shared_ptr};
        println!("atomic: {:?}, other: {:?}", &a.count, &a.count_a);
        thread::sleep(Duration::from_secs(2));
    }


    Ok(())
}


#[test]
fn align_size() {
    let size = std::mem::size_of::<SharedData>();
    println!("size is {}", size);
}


/// 单进程下, 原子操作
#[test]
fn test_atomic() {
    let atomic_i32 = Arc::new(AtomicI32::new(0));
    let a = atomic_i32.clone();
    let b = atomic_i32.clone();

    thread::spawn(move || {
        for i in 0..100000000 {
            a.fetch_sub(1, Ordering::SeqCst);
        }
        loop {
            println!("a: {:?}", &a);
            thread::sleep(Duration::from_secs(2));
        }
    });
    thread::spawn(move || {
        for i in 0..100000000 {
            b.fetch_add(1, Ordering::SeqCst);
        }
        loop {
            println!("b: {:?}", &b);
            thread::sleep(Duration::from_secs(2));
        }
    });

    thread::sleep(Duration::from_secs(100));
}


/// 共享内存, 在单进程多线程中, 可以用原子操作
///
/// 每个进程独享一个虚拟地址空间, 也许原子操作依赖它, 而不是依赖真实的内存.
///
/// Atomic不能做到多进程的原子操作, 那么究竟有没有办法做到多进程的原子操作呢.
#[test]
fn test_atomic3() -> anyhow::Result<()>{
    let size = std::mem::size_of::<SharedData>();

    let file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("rustix_mmap.bin").unwrap();
    file.set_len(size as u64);
    let borrowed_fd= file.as_fd();

    // // 创建共享内存映射 - 指针
    let shared_ptr = unsafe {
        mmap(
            ptr::null_mut(),
            size,
            ProtFlags::READ | ProtFlags::WRITE,
            MapFlags::SHARED,
            borrowed_fd,
            0,
        )? as *mut SharedData
    };
    let a = unsafe {&mut *shared_ptr};
    let a_arc = Arc::new(a);
    let b = a_arc.clone();
    let a = a_arc.clone();

    thread::spawn(move || {
        for i in 0..100000000 {
            a.count.fetch_sub(1, Ordering::SeqCst);
        }
        loop {
            println!("a: {:?}", &a);
            thread::sleep(Duration::from_secs(2));
        }
    });
    thread::spawn(move || {
        for i in 0..100000000 {
            b.count.fetch_add(1, Ordering::SeqCst);
        }
        loop {
            println!("b: {:?}", &b);
            thread::sleep(Duration::from_secs(2));
        }
    });

    thread::sleep(Duration::from_secs(100));
    Ok(())
}


/// 利用文件描述符, 尝试

#[test]
fn test_atomic2() -> anyhow::Result<()>{
    let op = env::args().collect::<Vec<_>>().get(3).unwrap_or(&"both".to_string()).clone();
    println!("{:#?}", op);
    let ptr = unsafe {
        rustix::mm::mmap_anonymous(
            std::ptr::null_mut(),
            4,
            ProtFlags::READ | ProtFlags::WRITE,
            MapFlags::SHARED | MapFlags::POPULATE,
        )
    }.unwrap() as *mut std::sync::atomic::AtomicI32;

    unsafe { (*ptr).store(0, std::sync::atomic::Ordering::SeqCst) };

    if op == "child" {
        let mut child = Command::new("cargo").args(["test", "--example", "rustix_mmap", "test_atomic2", "--", "--nocapture"]).spawn().unwrap();
        child.wait().unwrap();
    }
    thread::sleep(Duration::from_millis(100));

    let shared = unsafe { &*ptr };
    if op == "child" {
        for _ in 0..1_000_0000 {
            println!("into");
            shared.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    } else {
        for _ in 0..1_000_0000 {
            shared.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    loop {

        println!("Result: {}", shared.load(std::sync::atomic::Ordering::SeqCst));
        thread::sleep(Duration::from_secs(2));
    }

    Ok(())

}