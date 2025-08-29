use std::{env, thread};
use std::os::fd::{AsRawFd, FromRawFd};
use std::process::Command;
use std::time::Duration;
use rustix::event;
use rustix::io::{read, write};
use rustix::fs::flock;

/// 可用的标志位:
/// CLOEXEC;     执行时关闭 - 当子进程执行execve函数时, 自动关闭
/// NONBLOCK;    非阻塞模式
/// SEMAPHORE;   信号量模式 - 信号量模式指访问时减1, 默认会清零, 用作资源计数.
///
/// // 可以组合使用
/// let flags = event::EventfdFlags::CLOEXEC | event::EventfdFlags::NONBLOCK;
///
/// > 能不用信号量, 就不用信号量.
fn main() -> anyhow::Result<()> {
    process_eventfd()
    // thread_eventfd()
}

fn thread_eventfd() -> anyhow::Result<()> {
    // 1. 创建 eventfd
    let event_fd = event::eventfd(0, event::EventfdFlags::CLOEXEC)?;
    println!("Created eventfd: {:?}", event_fd);

    // 在另一个线程中发送通知
    let event_fd_clone = event_fd.try_clone()?;
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(20));
        println!("Thread: Sending notification...");
        write(&event_fd_clone, &1u64.to_ne_bytes()).unwrap();
    });

    // 主线程等待通知
    println!("Main: Waiting for notification...");
    let mut buffer = [0u8; 8];
    read(&event_fd, &mut buffer)?;

    let value = u64::from_ne_bytes(buffer);
    println!("Main: Received value: {}", value);
    Ok(())
}


fn process_eventfd() ->anyhow::Result<()> {
    let op = env::args().collect::<Vec<_>>().get(1).unwrap_or(&"both".to_string()).clone();

    match op.as_str() {
        _send @ "send" => {

            let fd_str = env::var("EVENTFD_FD")?;
            let fd: i32 = fd_str.parse()?;
            let event_fd = unsafe { std::fs::File::from_raw_fd(fd) };
            println!("Created eventfd: {:?}", event_fd);
            write(&event_fd, &1u64.to_ne_bytes())?;
        },
        _read @ "read" => {
            thread::sleep(Duration::from_millis(4));

            let fd_str = env::var("EVENTFD_FD")?;
            let fd: i32 = fd_str.parse()?;
            let event_fd = unsafe { std::fs::File::from_raw_fd(fd) };
            println!("Main: Waiting for notification...");
            let mut buffer = [0u8; 8];
            read(&event_fd, &mut buffer)?;
            let value = u64::from_ne_bytes(buffer);
            println!("Main: Received value: {}", value);
        },
        _both @ "both" => {
            let event_fd = event::eventfd(0, event::EventfdFlags::empty())?;
            let fd_num = event_fd.as_raw_fd().to_string();
            Command::new("cargo").arg("run").arg("--example").arg("rustix_eventfd").arg("send").env("EVENTFD_FD", fd_num.to_string()).spawn()?;
            Command::new("cargo").arg("run").arg("--example").arg("rustix_eventfd").arg("read").env("EVENTFD_FD", fd_num.to_string()).spawn()?;
        }
        _ => todo!()
    }
    Ok(())
}



