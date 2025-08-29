//!
//! 这个示例在尝试用内存共享实现一个进程锁,
//!
//! 在crate.io上基于共享内存实现的进程锁很少, 绝大多数是线程锁.
//!

use std::fs::OpenOptions;
use std::os::fd::AsFd;
use std::{env, fs, ptr, thread};
use std::any::Any;
use std::fmt::Debug;
use std::process::Command;
use std::ptr::write_volatile;
use std::sync::Arc;
#[allow(unused)]
use lock_api::{RawMutex, Mutex, GuardSend};
use std::sync::atomic::{fence, AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc::channel;
use std::time::Duration;
use lock_api::MutexGuard;
use rustix::mm::{mmap, MapFlags, ProtFlags};


#[derive(Debug)]
pub struct IpcMutexRaw(AtomicBool);
unsafe impl RawMutex for IpcMutexRaw {
    const INIT: IpcMutexRaw = IpcMutexRaw(AtomicBool::new(false));
    type GuardMarker = GuardSend;

    fn lock(&self) {
          while !self.try_lock() {
              println!("获取失败{:?}", self.0);
          }
      }

  fn try_lock(&self) -> bool {
      // println!("self.0 {:?}", self.0);
          self.0
              .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
              .is_ok()
      }

  unsafe fn unlock(&self) {
          self.0.store(false, Ordering::SeqCst);
      }
}

pub type IpcMutex<T> = lock_api::Mutex<IpcMutexRaw, T>;
pub type IpcMutexGuard<'a, T> = lock_api::MutexGuard<'a, IpcMutexRaw, T>;


#[repr(align(64))]  //  强制类型按指定字节数对齐 64 或者 64的倍数
#[repr(C)]  // 字段顺序按照声明顺序排列（Rust默认可能重排）
#[derive(Debug )]
struct Shm<T> {
    lock: AtomicBool,   // 1个字节, 但是在 repr(C)下是 4个字节
    _padding0: [u8; 3], // 填充4个字节, 与 Mutex 对齐
    data: T,    //
}

unsafe impl <T>  Sync for Shm<T> {}
unsafe impl <T> Send for Shm<T> {}

struct Data {}

struct Rutex<T> where T: Debug {
   inner : Mutex<IpcMutexRaw, T>,
}

impl <T> Rutex<T> where T: Debug + std::marker::Send + 'static {
    pub fn new<'a>(data : T) -> &'a mut Mutex<IpcMutexRaw, T> {
        let mut size = size_of_val(&data) + 8;
        while size % 64 != 0 { // 填充到 64 字节
            size += 1;
        }
        let file;

       file = OpenOptions::new().read(true).write(true).open("ipc_mem_lock").unwrap();

        println!("size: {}", size);
        file.set_len(size as u64).unwrap();
        let borrowed_fd= file.as_fd();

        let shared_ptr = unsafe {
            mmap(
                ptr::null_mut(),                        // 自动分配地址
                size,                                   // 共享内存的大小
                ProtFlags::READ | ProtFlags::WRITE, // 读和写对其他进程可见
                MapFlags::SHARED,                       // shared
                borrowed_fd,                            // 文件描述符
                0,                                 // 偏移量
            ).unwrap() as *mut Shm<T> as *mut Mutex<IpcMutexRaw, T>
        };

        let mutex = unsafe {
            let origion = &mut *shared_ptr;
            let d = Arc::new(&*shared_ptr);
            thread::spawn(move || {
                loop {
                    println!("3: {:?}", d);
                    thread::sleep(Duration::from_secs(2));
                }
            });
            let a = Shm { lock: Default::default(), _padding0: [0; 3], data: 0 };
            // println!("origin: {:?}", &origion.raw().0 as *const _);
            println!("origin: {:?}", shared_ptr);
            // println!("origin: {:?}", &origion.data as *const _);
            println!("origin: {:?}", size_of_val(origion));
            println!("shm: {:?}", &a as *const _);
            println!("shm: {:?}", &a.data as *const _);

            origion
        };

        // unsafe {
        //     std::ptr::copy_nonoverlapping(
        //         &data as *const T,
        //         shared_ptr.offset(8) as *mut T,
        //         size
        //     );
        // }

        // fs::remove_file("ipc_mem_lock").unwrap();
        let op = env::args().collect::<Vec<_>>().get(2).unwrap_or(&"".to_string()).clone();

        mutex
    }
}



fn lock_in_thread() {
    // let mu = Arc::new(Rutex::new(AtomicI32::new(0)));
    // let mu_clone1 = mu.clone();
    // let mu_clone2 = mu.clone();
    // let (send, receive) = channel::<bool>();
    // let send_1 = send.clone();
    // let send_2 = send.clone();
    // thread::spawn(move || {
    //     for i in 0..100000000 {
    //         let data = mu_clone1.lock();
    //         data.fetch_add(1, Ordering::Release);
    //     }
    //     send_1.send(true).unwrap();
    // });
    // thread::spawn(move || {
    //     for i in 0..100000000 {
    //         let data = mu_clone2.lock();
    //         data.fetch_sub(1, Ordering::Release);
    //     }
    //     send_2.send(true).unwrap();
    // });
    //
    // let mut c = 0;
    // loop {
    //     if let Ok(s) =  receive.recv() {
    //         c = c + 1;
    //         if c == 2{
    //             let a = mu.lock();
    //             println!("{:?}", a);
    //         }
    //     }
    // }
}

fn lock_in_processor() {
    let op = env::args().collect::<Vec<_>>().get(2).unwrap_or(&"".to_string()).clone();
    println!("op: {}", op);
    if fs::exists("ipc_mem_lock").unwrap() {
        fs::remove_file("ipc_mem_lock").unwrap();
    }
    let file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("ipc_mem_lock").unwrap();

    #[derive(Debug)]
    struct Message {
        count: i32
    }


        let message = Rutex::new(Message {count : 0});
        for i in 0..10 {
            let mut data = message.lock();
            if op == "child" {
                data.count -= 1;
            } else {
                data.count += 1;
            }
        }
        loop {
            // let mut data = message.lock();
            // unsafe { println!("2: {:?}", &message.raw().0 as *const _); }
            // let d =&message.data_ptr();
            // unsafe {
            //     println!("2: {:?}", &message.data_ptr());
            //     *message.data_ptr() = Message { count: 2};
            //     println!("{:?}", message);
            // }
            // message.lock.store(true, Ordering::SeqCst);
            // message.data.count = 6;
            // let a = message.lock();
            println!("2: {:?}", message);
            thread::sleep(Duration::from_secs(2));
        }

}

fn main() -> anyhow::Result<()>{
    let op = env::args().collect::<Vec<_>>().get(1).unwrap_or(&"".to_string()).clone();
    println!("op: {}", op);
    if op == "processor" {
        lock_in_processor();
    } else {
        lock_in_thread();
    }
    Ok(())
}




























