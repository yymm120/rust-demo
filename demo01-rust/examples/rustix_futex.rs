#![allow(unused)]
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::{env, ptr, thread};
use std::fs::OpenOptions;
use std::hint::spin_loop;
use std::num::NonZero;
use std::os::fd::AsFd;
use std::time::Duration;

/// Rustix 提供的 Futex 可用于实现进程同步
///
/// 而 rustix_futex_sync 是一个用 futex 实现的同步 API, 一般使用它就可以了.
///
/// Futex 可以让进程里的线程等待, 也可以唤起其他进程里的线程, 当配合共享内存使用时, 可以实现进程锁.
/// rustix_futex_sync 中的锁都是基于 Futex 实现.
///
/// Futex 的系统调用语法如下, 一般通过 rustix::thread::futex 去调用.
/// ```c
///        long syscall(
///                 SYS_futex,
///                 uint32_t                *uaddr,
///                 int                     futex_op,
///                 uint32_t                val,
///                 const struct timespec   *timeout,   /* or: uint32_t val2 */
///                 uint32_t                *uaddr2,
///                 uint32_t                val3
///         );
/// ```
use rustix::{thread::futex, process::*};
use rustix::mm::{mmap, MapFlags, ProtFlags};
use rustix_futex_sync::shm::Mutex;


/// 使用 Futex 的步骤如下:
///
/// 1. 使用AtomicU32作为监听对象. (可以是共享内存)
/// 2. 是否跨进程:
///     1. 如果要跨进程, 那么就声明一个共享内存, 不能用 PRIVATE
///     2. 如果不跨进程, 那么就用PRIVATE
///
/// 执行以下命令, 测试程序
/// ```
/// cargo run --example rustix_futex single
///
/// cargo run --example rustix_futex multi
/// ```
fn main() -> anyhow::Result<()> {
    let op = env::args().collect::<Vec<_>>().get(1).unwrap_or(&"both".to_string()).clone();
    if op == "single" {
        single_processor()?;
        return Ok(())
    } else if op == "multi" {
        multi_processor()?;
    } else if op == "bitset" {
        bitset_processor()?;
    }
    Ok(())
}


fn single_processor() -> anyhow::Result<()> {
    let count = AtomicU32::new(0);
    let count_arc = Arc::new(count);
    let count_arc_clone1 = count_arc.clone();
    let count_arc_clone2 = count_arc.clone();
    let count_arc_clone3 = count_arc.clone();


    thread::spawn(move || {
        // 休眠
        println!("线程1: 开始");
        while count_arc_clone1.load(Ordering::Acquire) == 0 {
            match futex::wait(
                &count_arc_clone1,      // 监听的原子
                futex::Flags::empty(),  // PRIVATE or REAL_TIME or empty, 如果需要跨进程可见, 不能选择PRIVATE作为参数
                0,                  // 期望值为 0, 如果相等, 就会令线程休眠, 如果不相等, 则会返回 Again Error.
                None                    // 超时时间, None 永不超时
            ) {
                Ok(_) => {println!("线程1: 被唤醒");}
                Err(_) => {println!("线程1: 值被改变, 重新检查!"); continue;}
            }
        }
        println!("线程1: 结束");
    });

    thread::spawn(move || {
        // 休眠
        println!("线程3: 开始");
        while count_arc_clone3.load(Ordering::Acquire) == 0 {
            match futex::wait(
                &count_arc_clone3,
                futex::Flags::empty(),
                0,
                None
            ) {
                Ok(_) => {println!("线程3: 被唤醒");}
                Err(_) => {println!("线程3: 值被改变, 重新检查!"); continue;}
            }
        }
        println!("线程1: 结束");
    });

    thread::spawn(move || {
        println!("线程2: 开始, 等待2s");
        thread::sleep(Duration::from_secs(2));
        println!("线程2: 修改标志位");
        count_arc_clone2.store(1, Ordering::Release);
        println!("线程2: 发送唤醒信号");
        // 需要设置要唤醒几个线程
        let woken = futex::wake(&count_arc_clone2, futex::Flags::empty(), 2).unwrap();
        println!("线程2: 唤醒了 {} 个线程", woken);
        // futex::wake(&count_arc_clone2, futex::Flags::empty(), 1).unwrap();
    });

    thread::sleep(Duration::from_secs(100));
    Ok(())


}
#[repr(C)]  // 字段顺序按照声明顺序排列（Rust默认可能重排）
#[repr(align(64))]  //  强制类型按指定字节数对齐 64 或者 64的倍数
#[derive(Debug )]
struct SharedData {
    count: AtomicU32,   // 4个字节
    count_a: i32,   // 4个字节
    _padding: [u8; 56], // 手动填充到64个字节
}

/// `cargo run --example rustix_futex multi`
///
/// 未使用共享内存的线程, 一个都没被唤醒
///
/// 使用了共享内存的线程，全部被唤醒.
///
/// 执行结果:
///
/// 进程740748 - 线程0: 已启动 - 准备休眠.
/// 进程740748 - 线程0: 已启动 - 准备休眠 (共享内存).
/// 进程740750 - 线程0: 已启动 - 准备休眠.
/// 进程740750 - 线程0: 已启动 - 准备休眠 (共享内存).
/// 进程740749 - 线程0: 已启动 - 准备休眠 (共享内存).
/// 进程740748 - 线程1: 已启动 - 准备休眠 (共享内存).
/// 进程740748 - 线程1: 已启动 - 准备休眠.
/// 进程740749 - 线程0: 已启动 - 准备休眠.
/// 进程740750 - 线程1: 已启动 - 准备休眠.
/// 进程740750 - 线程1: 已启动 - 准备休眠 (共享内存).
/// 进程740749 - 线程1: 已启动 - 准备休眠.
/// 进程740749 - 线程1: 已启动 - 准备休眠 (共享内存).
/// 进程740748 - 线程2: 已启动 - 准备休眠 (共享内存).
/// 进程740748 - 线程2: 已启动 - 准备休眠.
/// 进程740750 - 线程2: 已启动 - 准备休眠 (共享内存).
/// 进程740750 - 线程2: 已启动 - 准备休眠.
/// 进程740749 - 线程2: 已启动 - 准备休眠 (共享内存).
/// 进程740749 - 线程2: 已启动 - 准备休眠.
/// 进程740750 - 线程1: 被唤醒. (共享内存)
/// 进程740748 - 线程1: 被唤醒. (共享内存)
/// 进程740748 - 线程0: 被唤醒. (共享内存)
/// 进程740749 - 线程1: 被唤醒. (共享内存)
/// 进程740750 - 线程0: 被唤醒. (共享内存)
/// 进程740748 - 线程2: 被唤醒. (共享内存)
/// 进程740749 - 线程2: 被唤醒. (共享内存)
/// 进程740750 - 线程2: 被唤醒. (共享内存)
/// 进程740749 - 线程0: 被唤醒. (共享内存)
fn multi_processor() -> anyhow::Result<()> {
    let op = env::args().collect::<Vec<_>>().get(1).unwrap_or(&"both".to_string()).clone();

    let pid = getpid();
    let count = AtomicU32::new(0);
    let count_arc = Arc::new(count);
    let count_arc_clone1 = count_arc.clone();
    let count_arc_clone2 = count_arc.clone();


    let size = size_of::<SharedData>();


    // 起 4 个进程
    if op == "multi" {
        if std::fs::exists("rustix_futex.bin")? {
            std::fs::remove_file("rustix_futex.bin")?;
        }

        let file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("rustix_futex.bin")?;
        file.set_len(size as u64);
        println!("{}",  std::fs::exists("rustix_futex.bin")?);
        thread::sleep(Duration::from_secs(2));
        for i in 0..3 {
            let child = Command::new("cargo").args(["run", "--example", "rustix_futex", "child"]).spawn();
        }
        /**
         * 在父进程中唤醒, 子进程压根没有反应, 因为Atomic都不是用的同一个.
         * 无法唤醒
         */
        thread::sleep(Duration::from_secs(2));
        count_arc_clone2.fetch_add(1, Ordering::Release);
        let woken = futex::wake(&count_arc_clone2, futex::Flags::empty(), 2).unwrap();

        /**
           操作共享内存的原子类型, 可以唤醒所有其他进程中的线程
        */
        thread::sleep(Duration::from_secs(2));
        let file = OpenOptions::new().read(true).write(true).open("rustix_futex.bin")?;
        let borrowed_fd= file.as_fd();

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
        let data = Arc::new(data);
        // 这里唤起指定个等待线程, 按照 man 手册中说明, 这里应该填 u32::MAX 唤起所有线程
        // 然而u32::MAX 实际上只唤醒了1个线程, 也许是个 bug
        let woken = futex::wake(&data.count, futex::Flags::empty(), u8::MAX as u32).unwrap();

        ///
        thread::sleep(Duration::from_secs(100));
        return Ok(())
    } else if op == "child" {

        thread::sleep(Duration::from_secs(2));

        while !std::fs::exists("rustix_futex.bin")? {
            std::hint::spin_loop();
            println!("not exists");
        }
        let file = OpenOptions::new().read(true).write(true).open("rustix_futex.bin")?;
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
        let data = Arc::new(data);


        // 每个进程起 3 个线程
        for i in 0..3 {
            let msg = format!("进程{} - 线程{}: 被唤醒.", pid.as_raw_nonzero(), i);
            let count_arc_clone1 = count_arc.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(i * 100));
                println!("进程{} - 线程{}: 已启动 - 准备休眠.", pid.as_raw_nonzero(), i);
                thread::sleep(Duration::from_millis(i * 100));
                while count_arc_clone1.load(Ordering::Acquire) == 0 {
                    match futex::wait(
                        &count_arc_clone1,      // 监听的原子
                        futex::Flags::empty(),  // PRIVATE or REAL_TIME or empty, 如果需要跨进程可见, 不能选择PRIVATE作为参数
                        0,                  // 期望值为 0, 如果相等, 就会令线程休眠, 如果不相等, 则会返回 Again Error.
                        None                    // 超时时间, None 永不超时
                    ) {
                        Ok(_) => {println!("{}", msg);}
                        Err(_) => {println!("值被改变, 重新检查!"); continue;}
                    }
                }
            });
        }

        // 每个进程起 3 个线程
        for i in 0..3 {
            let msg = format!("进程{} - 线程{}: 被唤醒. (共享内存)", pid.as_raw_nonzero(), i);
            let d = data.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(i * 100));
                println!("进程{} - 线程{}: 已启动 - 准备休眠 (共享内存).", pid.as_raw_nonzero(), i);
                thread::sleep(Duration::from_millis(i * 100));
                while d.count.load(Ordering::Acquire) == 0 {
                    match futex::wait(
                        &d.count,      // 监听的原子
                        futex::Flags::empty(),  // PRIVATE or REAL_TIME or empty, 如果需要跨进程可见, 不能选择PRIVATE作为参数
                        0,                  // 期望值为 0, 如果相等, 就会令线程休眠, 如果不相等, 则会返回 Again Error.
                        None                    // 超时时间, None 永不超时
                    ) {
                        Ok(_) => {println!("{}", msg);}
                        Err(_) => {println!("值被改变, 重新检查!"); continue;}
                    }
                }
            });
        }

        thread::sleep(Duration::from_secs(100));

    }


    Ok(())
}



/// 要注意: Bitset并不是对比较值做掩码操作, 而是将wait和wake进行对应.
///
/// 比如说: 最后一个参数为 0000...0001 的wake只能唤醒最后一个参数为0000....0001的wait, 而不能唤醒0000....0010的wait.
///
/// 至于val期待值， 只要与原值不同就可以跳出循环, 与唤醒不唤醒无关, 这个可以自行控制.
///
fn bitset_processor() -> anyhow::Result<()> {
    let op = env::args().collect::<Vec<_>>().get(2).unwrap_or(&"both".to_string()).clone();

    let file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("rustix_futex.bin")?;

    thread::sleep(Duration::from_secs(1));

    let size = size_of::<SharedData>();
    file.set_len(size as u64);
    let borrowed_fd= file.as_fd();
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
    let data = Arc::new(data);
    let data_clone = data.clone();
    // let d3 = d.clone();
    if op == "1" {
        for i in 0..3 {
            // let child = Command::new(cargo").args(["run", "--example", "rustix_futex", "multi", ""]).spawn();
        }
    } else if op == "2" {

        for i in 0..3 {
            let d1 = data.clone();
            thread::spawn(move || {
                println!("线程 {i} 正在等待被唤醒.");
                while d1.count.load(Ordering::Acquire) == 0 {
                    spin_loop();
                    match futex::wait_bitset(&d1.count, futex::Flags::empty(), 0b0000_0000_0000_0000, None, NonZero::new(0b0000_0000_0000_0000_0001).unwrap()) {
                        Ok(_) => {
                            println!("唤醒 {i} 成功.")
                        }
                        Err(e) => {
                            eprintln!("发生错误: {e}");
                        }
                    }
                }
                println!("线程 {i} 跳出循环");
                // thread::sleep(Duration::from_secs(1));
            });
        }
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(3));
            // 只要存入不为0的数, 就能使等待线程跳出循环
            &data_clone.count.store(0b0000_0000_0000_0000_0001, Ordering::Release);
            // 这里只做唤醒, 不做条件控制
            let woken = futex::wake_bitset(&data_clone.count, futex::Flags::empty(), 3,  NonZero::new(0b1000_0000_0000_0000_0001).unwrap()).unwrap();

        });
    }

    thread::sleep(Duration::from_secs(20));


    Ok(())
}


fn futex_wait() {

}

