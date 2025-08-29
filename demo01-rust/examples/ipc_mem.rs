#![allow(unused)]
use std::{env, thread};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::marker::PhantomData;
use std::os::fd::{AsFd, AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::{read_volatile, write_volatile};
use std::sync::atomic::{fence, AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use bincode::{Decode, Encode};
use memmap2::{Mmap, MmapMut};
use rustix::event;

#[repr(C)]
#[derive(Debug)]
struct MyData {
    read_pending: AtomicBool,
    counter: i32,
    temperature: f32,
    data: String,
    active: bool
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let op = env::args().collect::<Vec<_>>().get(1).unwrap_or(&"both".to_string()).clone();

    match op.as_str() {
        write @ "write" => {
            run_write()?;
        },
        read @ "read" => {
            run_read().await?;
        },
        both @ "both" => {
            run_both()?;
        }
        _ => todo!()
    }

    Ok(())
}

fn run_both() -> anyhow::Result<()> {
    let path = Path::new("ipc_mem.bin");
    if std::fs::exists(path)? {
        std::fs::remove_file(path)?;
    }

    let file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("ipc_mem.bin")?;

    file.set_len(size_of::<MyData>() as u64).unwrap();

    let mut c1 = Command::new("cargo").arg("run").arg("--example").arg("ipc_mem").arg("--").arg("write").env("TEMP_MEM_FILE", path).spawn()?;
    let mut c2 = Command::new("cargo").arg("run").arg("--example").arg("ipc_mem").arg("--").arg("read").env("TEMP_MEM_FILE", path).spawn()?;
    // run_read()?;

    c1.wait().unwrap();
    c2.wait().unwrap();

    Ok(())
}

#[repr(C)]
#[repr(align(64))]
#[derive(Encode, Decode, Debug)]
struct RingBuffer {
    data: [u8; u8::MAX as usize],
    head: AtomicBool,
    tail: AtomicBool,
    event_num: Option<i32>,
    count: AtomicI32,
    a_count: i32,
}

impl RingBuffer {
    pub fn new() -> Self {
        Self {
            data: [0u8; u8::MAX as usize],
            head: AtomicBool::new(false),
            tail: AtomicBool::new(false),
            event_num: None,
            count: AtomicI32::new(0),
            a_count: 0,
        }
    }
}

fn run_write() -> anyhow::Result<()> {
    let path = Path::new("ipc_mem.bin");
    if std::fs::exists(path)? {
        std::fs::remove_file(path)?;
    }
    let event_fd = event::eventfd(0, event::EventfdFlags::empty())?;
    let fd_num: i32 = event_fd.as_raw_fd().to_string().parse()?;

    println!("{:?}", fd_num);

    let r_buf = RingBuffer::new();

    println!("{:?}", size_of::<RingBuffer>() as u64);

    env::set_var("TEMP_MEM_FILE", "ipc_mem.bin");
    let file_path = env::var("TEMP_MEM_FILE")?;

    let file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("ipc_mem.bin")?;

    // 初始化数据不能为Null, 长度必须小于isize::MAx
    file.set_len(size_of::<RingBuffer>() as u64).unwrap(); // 是字节长度, 而非Bit

    let mut mmap = unsafe { MmapMut::map_mut(&file)?};

    let mut shared_data = unsafe {
        println!("{:?}", mmap.as_mut_ptr());
        &mut *(mmap.as_mut_ptr() as *mut RingBuffer) // 映射300~500ns之间, 极少数达到 1us
    };

    use std::ptr;

    // 强制直接内存访问，绕过缓存
    unsafe fn volatile_write(data: *mut u8, value: u8) {
        ptr::write_volatile(data, value);
    }

    let a = RingBuffer::new();
    shared_data.event_num = Some(fd_num);

    // thread::sleep(Duration::from_secs(5));
    // let mut a = bincode::encode_into_slice(a, shared_data, bincode::config::standard())?;

    // shared_data = ;

    // 必须赋值, 不能为Null
    // shared_data.counter = 43;
    // shared_data.temperature = 43.0;
    // shared_data.active = false;
    // shared_data.data = "100000000000".to_string();
    let mut i = 0;

    loop {
        if i == 100000000 {
            break
        }
        fence(Ordering::Acquire);
        shared_data.count.fetch_sub(1, Ordering::SeqCst);
        fence(Ordering::Release);
        i = i + 1;
    }
    let mut i = 0;
    loop {
        if i == 100000000 {
            break
        }
        fence(Ordering::Acquire);
        shared_data.a_count = shared_data.a_count - 1;
        fence(Ordering::Release);
        i = i + 1;
    }


    loop {
        let mut s_d = unsafe {
            println!("{:?}", mmap.as_mut_ptr());
            let p = &*(mmap.as_mut_ptr() as *const RingBuffer);
            let st = Instant::now();
            let res = read_volatile(p);
            let a = st.elapsed();
            println!("red {:#?}", a);
            res
        };
        fence(Ordering::Acquire);
        println!("写入结构体: {:?}", s_d);
        thread::sleep(Duration::from_secs(2));
    }
    if std::fs::exists(path)? {
        std::fs::remove_file(path)?;
    }
    Ok(())
}


async fn run_read() -> anyhow::Result<()> {
    // 必须先初始化
    let path = Path::new("ipc_mem.bin");
    while !std::fs::exists(path)? {
        std::hint::spin_loop();
        println!("not exists");
    }
    let file = OpenOptions::new().read(true).write(true).open("ipc_mem.bin")?;
    while file.metadata()?.len() == 0 {
        println!("does not data");
        std::hint::spin_loop();
    }

    let mut mmap = unsafe {
        match MmapMut::map_mut(&file) {
            Ok(m) => {m}
            Err(e) => {
                panic!("map failed: {:#?}", e);
            }
        }
    };

    let mut shared_data = unsafe {

        println!("{:?}", mmap.as_mut_ptr());
        let st = Instant::now();
        let res = &mut *(mmap.as_mut_ptr() as *mut RingBuffer);
        let a = st.elapsed();
        println!("{:#?}", a);
        res
    };


    while shared_data.event_num == None {
        println!("还没有准备好通信")
    }
    // write_volatile()
    // let r1 = unsafe { read_volatile(&i as _) };
    // let r2 = unsafe { read_volatile(&a[r1 as usize] as _) };
    let fd_num = shared_data.event_num.unwrap();
    let event_fd = unsafe { std::fs::File::from_raw_fd(fd_num) };

    println!("准备好了");

    let mut i = 0;

    loop {
        if i == 100000000 {
            break
        }
        fence(Ordering::Acquire);
        shared_data.count.fetch_add(1, Ordering::SeqCst);
        fence(Ordering::Release);
        i = i + 1;
    }
    let mut i = 0;
    loop {
        if i == 100000000 {
            break
        }
        fence(Ordering::Acquire);
        shared_data.a_count = shared_data.a_count + 1;
        fence(Ordering::Release);
        i = i + 1;
    }


    loop {
        let mut s_d = unsafe {
            println!("{:?}", mmap.as_mut_ptr());
            let p = &*(mmap.as_mut_ptr() as *const RingBuffer);
            let st = Instant::now();
            let res = read_volatile(p);
            let a = st.elapsed();
            println!("red {:#?}", a);
            res
        };
        fence(Ordering::Acquire);
        println!("写入结构体: {:?}", s_d);
        thread::sleep(Duration::from_secs(2));
    }


    Ok(())
}