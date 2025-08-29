
use std::sync::Arc;
use std::{env, thread};
use std::time::Duration;
use crate::mutex::{cleanup,};

pub mod mutex {
    use std::{fs, ptr, thread};
    use std::ffi::c_void;
    use std::fmt::{Debug, Display, Formatter};
    use std::fs::{File, OpenOptions};
    use std::hint::spin_loop;
    use std::io::Read;
    use std::os::fd::AsFd;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{   AtomicU32,   Ordering,} ;
    use std::thread::yield_now;
    use std::time::Duration;
    use lazy_static::lazy_static;
    use libc::{c_int, sighandler_t};
    use lock_api::{RawMutex, Mutex, GuardSend, MutexGuard};
    use rustix::{fs::flock, mm::{mmap, munmap, MapFlags, ProtFlags}};
    use rustix::fs::FlockOperation;
    use rustix::thread::futex;
    use sysinfo::{System};


    pub struct EmptyMemory();

    #[derive(Debug)]
    pub struct MemoryPtr<T: 'static>(*const SharedMemory<T>);
    #[derive(Debug)]
    pub struct MemoryPtrMut<T: 'static>(*mut SharedMemory<T>);
    unsafe impl <T> Sync for MemoryPtr<T> {}
    unsafe impl <T> Send for MemoryPtr<T> {}

    lazy_static! {
        static ref OLD_HANDLERS: std::sync::Mutex<Vec<(c_int, libc::sigaction)>> = std::sync::Mutex::new(Vec::new());
        // 只需要元数据, 所以用EmptyMemory作为类型, 不代表内存中没有数据
        static ref SHARED_MEMORIES: std::sync::Mutex<Vec<(PathBuf, &'static mut SharedMemory<EmptyMemory>)>> = std::sync::Mutex::new(Vec::new());
    }

    const EXIT_SIGNALS: [c_int; 14] = [
        libc::SIGHUP,    // 1 - 终端挂断
        libc::SIGINT,    // 2 - 中断 (Ctrl+C)
        libc::SIGQUIT,   // 3 - 退出 (Ctrl+\)
        libc::SIGILL,    // 4 - 非法指令
        libc::SIGABRT,   // 6 - 异常中止
        libc::SIGFPE,    // 8 - 浮点异常
        libc::SIGSEGV,   // 11 - 段错误
        libc::SIGBUS,    // 7 - 总线错误
        libc::SIGPIPE,   // 13 - 管道破裂
        libc::SIGALRM,   // 14 - 闹钟超时
        libc::SIGTERM,   // 15 - 终止信号
        libc::SIGXCPU,   // 24 - CPU时间超限
        libc::SIGXFSZ,   // 25 - 文件大小超限
        libc::SIGSYS,
        // 注意：SIGKILL (9) 和 SIGSTOP (19) 无法被捕获
    ];

    extern "C" fn handle_all_signals(
        sig: c_int,
        info: *mut libc::siginfo_t,
        context: *mut c_void
    ) {
        cleanup();
        let old_handlers = OLD_HANDLERS.lock().unwrap();
        for &(old_sig, ref old_sig_action) in old_handlers.iter() {
            if old_sig == sig {
                unsafe {
                    let handler = old_sig_action.sa_sigaction;

                    // 检查是否是有效的处理器
                    if handler != 0 && handler != libc::SIG_DFL as usize && handler != libc::SIG_IGN as sighandler_t {
                        // 调用旧的处理器
                        let handler_func: extern "C" fn(c_int, *mut libc::siginfo_t, *mut c_void) = std::mem::transmute(handler);
                        handler_func(sig, info, context);
                        break;
                    } else {
                        break;
                    }
                }
            }
        }
        std::process::exit(0);
    }


    pub fn cleanup() {
        let shared_memories = SHARED_MEMORIES.lock().unwrap();
        let pid = rustix::process::getpid().as_raw_nonzero().get();
        for memory in shared_memories.iter() {
            {
                let mut rpv_guard = memory.1.rpv.lock();
                for (i, p) in rpv_guard.iter_mut().enumerate() {
                    if *p == pid {
                        *p = 0;
                    }
                }
            }
            // 这里有可能有问题, 如果其他用户刚好在这时创建文件, 那么有概率被错误的删除.
            // 所以我把STATUS 设置为 PENDING, 让新加入访问队列的进程先等待, 直到删除完成后再唤醒对方
            // 即使删除了文件, 也可以成功将对方唤醒, 因为共享内存不会因为文件被删除而失去作用. 匿名共享内存甚至不需要文件.
            if memory.1.is_empty_rpv() {
                memory.1.inner_status().store(PENDING, Ordering::SeqCst);
                if fs::exists(&memory.0).unwrap_or(false) {
                    let _ = fs::remove_file(&memory.0);
                    memory.1.inner_status().store(INITIAL, Ordering::SeqCst);
                    let _ = futex::wake(&memory.1.status, futex::Flags::empty(), u8::MAX as u32).unwrap();
                }
            }
        }
    }

    #[derive(Debug )]
    #[repr(align(64))]  //  强制类型按指定字节数对齐 64 或者 64的倍数
    pub struct Rutex<T> where T: Sized + 'static {
        ptr: MemoryPtr<T>,
        inner: &'static SharedMemory<T>,
        // inner_data: &'a T,
        // inner_lock: &'a mut Mutex<IpcMutexRaw, T>,
        size: usize,
        // is_creator: bool,
        path: PathBuf,
        file: File,
    }

    const PENDING: u32 = 1;
    const CREATED: u32 = 2;
    const INITIAL: u32 = 0;

    impl <T: Debug + 'static> Rutex<T> {

        /// 使用方式与普通Mutex相同, 唯一的区别是, 必须额外指定一个文件路径, 就像socket通信必须指定socket文件, TCP通信必须指定Host:Port一样.
        /// 基于共享内存的锁, 就必须指向同一块内存, 对于mmap来说, 就是同一个文件路径, 用`path`来说明, 这很好理解.
        ///
        /// 第二个参数并非初始数据, 只是需要传入一个值, 方便计算占用空间, 避免内存浪费. 也许以后会改成 `len`
        ///
        /// Example:
        /// ```
        /// let rutex = Rutex::new("shared-memory-demo.bin".as_ref(), 0);
        /// {
        ///     let data = rutex.lock();
        ///     *data = 1;
        /// }
        /// ```
        ///
        /// 注意:
        ///
        /// path 会用来创建一个存在的`shared-memory-demo.bin`文件, 在创建时会清空其中的内容, 如果每个进程都进行清空, 会出现初始数据不一致的问题.
        /// 所以竞争从创建共享内存文件的时候就开始了, 解决办法是: 每个进程都先做内存映射, 通过检查其中的标志位来判断内存是否干净, 如果 `STATUS`标志位为 `INITIAL`, 所有进程都会尝试对标志位更新为`PENDING`, 然后更新为`CREATED`, 只有更新成功的进程被认为是创建者.
        ///
        /// - [INITIAL]: 数据不可用, 需要竞争出一个创建者, 来对共享内存中的数据初始化.
        /// - [PENDING]: 数据不可用, 创建者正在对共享内存中的数据初始化, 通常是用`truncate`函数清空数据, 并设置文件大小.
        /// - [CREATED]: 数据可用. 表明数据已经初始化完成.
        ///
        /// 创建者负责:
        /// 1. 创建或读取文件, 如果文件里有内容, 应该执行清理. (也许以后, 应该让使用者决定是否清理)
        /// 2. 创建内存映射, 并向其中添加初始化数据.
        ///
        /// 注意: 任何进程/线程传入的initial数据都有可能在此刻被覆盖. 因此, 第二个参数[T]只用来计算了一下`size`, 不会实际写入内存. 在数据同步之前, 只会将数据全部写0. (也许它不应该叫`initial`).
        ///
        /// FYI: 之所以使用 `mmap` 做共享内存, 是因为基于文件的内存共享方便调试. (除了`mmap`, 还可以通过 system v 实现内存映射).
        /// 在linux上, 我们可以用以下命令实施查看Hex数据.
        /// ```
        /// watch -n 1 'xxd 'shared_memory.bin | tail -5'
        /// ```
        ///
        /// 出现死锁的原因: 基于共享内存的锁, 死锁原因大概率是起始数据不一致. (资源泄露)
        ///
        /// - 一般来说, 当所有锁释放时, 共享内存应该被清理掉, 但如果程序非正常关闭, 例如遭遇`kill -9` 命令, drop函数可能不会执行, 如此一来, 共享内存文件将一直存在.
        /// - 我已经使用了 RAII / Signal Ctrl+C / Signal Kill 三种方式去删除共享内存文件, 理论上不容易出现资源泄露的情况. 详见[Rutex::drop]
        /// - 并且加了一层保护措施: 在 Mutex::new 的时候检查 Rpv 状态
        ///
        /// Rpv 用来存放所有正在使用共享内存的 pid, 当下一个使用者 new 的时候, 检查并校准Rpv.
        ///
        pub fn new(path: &Path, initial: T) -> Self {

            let pid = rustix::process::getpid().as_raw_nonzero();
            let mut file = OpenOptions::new().read(true).create(true).write(true).open(&path).unwrap_or_else(|e| panic!("Could not open {:?}: {}", &path, e));
            let borrowed_fd= file.as_fd();


            let meta_len = file.metadata().unwrap().len();


            let size = if meta_len == 0 {             // 计算共享内存的尺寸
                let mut size = size_of_val(&initial) ;
                while size < (8 + 68 + 4) || size % 64 != 0 { // 填充到 64 字节的倍数
                    size += 1;
                }
                size
            } else {
                meta_len as usize
            };

            let _ = file.set_len(size as u64);

            // 创建共享内存映射 - 得到虚拟空间内存地址 - 指针
            let shared_ptr = unsafe {
                mmap(
                    ptr::null_mut(),                            // 自动分配地址
                    size,                                       // 共享内存的大小
                    ProtFlags::READ | ProtFlags::WRITE,    // 读和写对其他进程可见
                    MapFlags::SHARED,                           // shared
                    borrowed_fd,                                // 文件描述符
                    0,                                    // 偏移量
                ).unwrap_or_else(|e| {
                    flock(borrowed_fd, FlockOperation::Unlock).unwrap();
                    panic!("Mapping memory failed: pid {} {:?}", pid, e)
                })
                    as *const SharedMemory<T> as *mut SharedMemory<T>
            };

            // 如果是是创建者, 清空文件, 否则阻塞等待唤醒
            let memory_data = unsafe {&mut *shared_ptr};

            if !memory_data.is_aligned() {
                panic!("Memory pointer is not aligned."); // 这段代码应该永远不会执行.
            }


            println!("memory data: {:?}", size_of_val(memory_data));


            let mut system = System::new_all();
            system.refresh_all();
            let pids = system.processes().keys()
                .map(|pid| pid.as_u32() as i32)
                .collect::<Vec<_>>();
            unsafe {
                let mut rpv_pids = &mut *memory_data.rpv.data_ptr();
                for (i, g)  in rpv_pids.iter_mut().enumerate() {
                    if !pids.contains(&g) {
                        *g = 0
                    }
                }
            }



            let mut count = 500;
            loop {
                let status = memory_data.status.load(Ordering::Acquire);
                let is_empty_rpv = memory_data.is_empty_rpv();
                if is_empty_rpv || status != CREATED {
                    if status == PENDING {
                        if count <= 6 { // 6次失败, 然后 panic
                            match futex::wait(&memory_data.inner_status(), futex::Flags::empty(), status, None)  { // 不设置超时时间
                                Ok(_) => {/*  */ println!("已唤醒");}
                                Err(e) => {
                                    if count <= 0 {
                                        panic!("Mapping memory failed: pid {} {:?}", pid, e);
                                    }
                                }
                            }
                        }
                    } else {
                        match memory_data.status.compare_exchange(status, PENDING, Ordering::Release, Ordering::Acquire) {
                            Ok(_) => {
                                // 清空文件
                                file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open(&path).unwrap_or_else(|e|{
                                    // panic之前, 必须将其他进程唤醒, 否则其他进程都会中毒.
                                    memory_data.status.store(INITIAL, Ordering::Release);
                                    let _ = futex::wake(&memory_data.inner_status(), futex::Flags::empty(), u8::MAX as u32).unwrap();
                                    panic!("Could not open {:?}: {}", &path, e);
                                });
                                // is_creator = true;

                                println!("into");
                                let mut buf = Vec::new();
                                let a = file.read_to_end(&mut buf).unwrap();
                                println!("buf: {:?}", buf);
                                println!("size: {}", size);
                                let _ = file.set_len(size as u64);
                                memory_data.status.store(CREATED, Ordering::Release);

                                // 最多唤醒 u8::MAX 个线程, 应该是够了。
                                let _ = futex::wake(&memory_data.inner_status(), futex::Flags::empty(), u8::MAX as u32).unwrap();

                                break
                            }
                            Err(_) => {count -= 1;
                            }
                        }
                    }


                    if count <= 400 { // 100次 高强度 CAS
                        yield_now();
                    }

                    count -= 1;
                    spin_loop(); // CAS自旋优化
                } else {
                    break;
                }
            }

            memory_data.increment_strong_count();

            unsafe {
                let sig_action = libc::sigaction {
                    sa_sigaction: handle_all_signals as libc::sighandler_t,
                    sa_mask: std::mem::zeroed(),
                    sa_flags: libc::SA_SIGINFO,
                    sa_restorer: None,
                };

                let mut old_action: libc::sigaction = core::mem::zeroed();
                for &sig in EXIT_SIGNALS.iter() {
                    libc::sigaction(sig, &sig_action, &mut old_action as *mut libc::sigaction);
                    let mut handlers = OLD_HANDLERS.lock().unwrap();
                    handlers.push((sig, old_action))
                }
            }

            unsafe {
                let mut shared_memories = SHARED_MEMORIES.lock().unwrap();
                let m = &mut * (shared_ptr as *mut SharedMemory<EmptyMemory>);
                shared_memories.push((path.into(), m));
            }

            // 必须保证裸指针不会被清理掉, 所以需要一个具有所有权的对象, 用来存放裸指针. (也许有更好的办法)
            Self {
                ptr: MemoryPtr(shared_ptr),
                inner: memory_data,
                size,
                // is_creator: false,
                path: path.to_path_buf(),
                file,
            }
        }
    }

    /// RAII 可能不够用, 任何形式的退出都应该清理文件, 清理内存映射.
    ///
    /// - ctrl c: 用 signal 实现退出时清理文件 - 需要依赖 libc, 因为rustix没有实现 sigaction.
    /// - kill: 用 signal 实现退出时清理文件 - 需要依赖 libc, 因为rustix没有实现 sigaction.
    /// - RAII: 用 Drop trait 实现清理文件
    impl <T> Drop for Rutex<T> {
        fn drop(&mut self) {
            // 将自己从共享内存中移除, 然后取消内存映射
            // 如果自己已经是最后一个被移除的线程, 那么要将文件删除.

            let _ = &self.inner.drop();

            let mut system = System::new_all();
            system.refresh_all();
            let pids = system.processes().keys()
                .map(|pid| pid.as_u32() as i32)
                .collect::<Vec<_>>();
            let mut guard = self.inner.rpv.lock();
            let pid = rustix::process::getpid().as_raw_nonzero().get();
            for (i, g)  in guard.into_iter().enumerate() {
                if !pids.contains(&g) {
                    guard[i] = 0
                }
                if g == pid {
                    guard[i] = 0
                }
            }

            if self.inner.is_empty_rpv() {

            }

            unsafe {
                munmap(self.ptr.0 as *mut c_void, self.size).unwrap();
            }
        }
    }

    impl <T> Rutex<T> {

        pub fn inner_lock(&self) -> &Mutex<IpcMutexRaw, T> {
            &self.inner.lock
        }

        pub fn inner_data(&self) -> &'static T {
            self.inner.inner_data()
        }

        pub fn lock(&self) -> MutexGuard<'static, IpcMutexRaw, T> {
            self.inner.lock.lock()
        }

        pub fn try_lock(self) {
            self.inner.lock.try_lock();
        }
        pub fn is_locked(&self) -> bool {
            self.inner.lock.is_locked()
        }
    }

    impl <T> Rutex<T> {
        pub fn path(&self) -> &Path {
            &*(self.path)
        }

        pub fn file(&self) -> &File {
            &self.file
        }
    }


    type RpvMutex = Mutex<IpcMutexRaw, [i32; 16]>;

    #[derive(Debug )]
    #[repr(align(64))]            // 强制类型按指定字节数对齐 64 或者 64的倍数
    #[repr(C)]                    // 字段顺序按照声明顺序排列（Rust默认可能重排）
    struct SharedMemory<T>
    where T: Sized + 'static {    // 确保数据的生命周期足够长, 且与进程无关. 确保数据的长度已知.
        status: AtomicU32,        // 4个字节
        _pad: [u8; 4],
        rpv: RpvMutex,             // 4 + 64 个字节, 引用计数, 同时表示允许多少个进程进入等待队列
        lock: Mutex<IpcMutexRaw, T>, // 4个字节 + T 类型的字节数
    }

    /// T 数据指针偏移: 1 + 68 +  4 = 73
    // const LOCK_OFFSET: isize = 73;
    // const RPV_OFFSET: isize = 1;
    // const STATUS_OFFSET: isize = 0;

    unsafe impl <T> Sync for SharedMemory<T> {}
    unsafe impl <T> Send for SharedMemory<T> {}

    impl <T> Display for SharedMemory<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }


    impl <T> SharedMemory<T> {

        pub fn inner_data(&self) -> &'static T {
            unsafe {
                let ptr = self.lock.data_ptr();
                &*ptr
            }
        }
        #[allow(unused)]
        pub fn inner_data_mut(self) -> &'static mut T {
            unsafe {
                let mut ptr = self.lock.data_ptr();
                &mut *ptr
            }
        }

        pub fn is_aligned(&self) -> bool {
            let ptr: _ = self as *const SharedMemory<T>;
            ptr.align_offset(align_of::<SharedMemory<T>>()) == 0
        }

        #[allow(unused)]
        pub fn into_lock(self) ->  Mutex<IpcMutexRaw, T> {
            self.lock
        }

        pub fn inner_rpv(&self) -> &RpvMutex {
             &self.rpv
        }

        pub fn inner_status(&self) -> &AtomicU32 {
            &self.status
        }

        pub fn is_empty_rpv(&self) -> bool{
            let ptr = self.inner_rpv() ;
            match ptr.try_lock() {
                None => unsafe {
                    let data = &*self.inner_rpv().data_ptr();
                    if data.iter().all(|&x| x == 0) {
                        ptr.force_unlock();
                        true
                    } else {
                        false
                    }
                }
                Some(guard) => {
                    !guard.iter().any(|m| *m != 0)
                }
            }
        }

        pub fn decrement_strong_count(&self) {
            let guard = self.inner_rpv().lock();
            let pid = rustix::process::getpid().as_raw_nonzero().get();
            guard.iter().enumerate().for_each(|(i, p)| {
                if pid == *p {
                    unsafe {
                        let ptr = &*guard as *const _ as *mut [i32; 16];
                        let rpv_pid = &mut *(ptr.byte_offset((i * 4) as isize) as *mut i32);
                        *rpv_pid = 0
                    }
                }
            })
        }

        pub fn increment_strong_count(&self) {
            let mut guard = self.inner_rpv().lock();
            let pid = rustix::process::getpid().as_raw_nonzero().get();
            if !guard.iter().any(|p| pid == *p) {
                let zero_idx = guard.iter().enumerate().filter_map(|(i, p)| {
                    if *p == 0 {
                        Some(i)
                    } else {None}
                }).collect::<Vec<_>>();
                if zero_idx.len() > 0 {
                    guard[zero_idx[0]] = pid;
                } else {
                    panic!("Watching Queue already fulled, total {}, Shared Memory using in these processors: {:?}", guard.len(), guard);
                }
            }
        }

        fn inner_ptr(&self) -> MemoryPtr<T> {
            MemoryPtr(self as *const SharedMemory<T>)
        }

        fn inner_mut(&self) -> MemoryPtrMut<T> {
            MemoryPtrMut(self as *const SharedMemory<T> as *mut SharedMemory<T>)
        }

        /// 删除文件应该交给使用ShareMemory的角色.
        /// 当引用计数为0时, 应该清空所有数据, 这里全部填充0.
        pub fn drop(&self) {
            self.decrement_strong_count();

            if self.is_empty_rpv() {
                unsafe {
                    ptr::write_bytes(self.inner_mut().0, 0, size_of_val(&self)); // 清空, 全部写0
                }
            }
        }
    }

    const LOCKED: u32 = 1;
    const UN_LOCKED: u32 = 0;
    #[derive(Debug)]
    pub struct IpcMutexRaw(AtomicU32);
    unsafe impl RawMutex for IpcMutexRaw {
        const INIT: IpcMutexRaw = IpcMutexRaw(AtomicU32::new(0)); // 这行代码不会生效, 因为AtomicU32来自于共享内存, 而非 new .
        type GuardMarker = GuardSend;

        fn lock(&self) {
            let mut count = 500;
            while !self.try_lock() {
                if count < 400 {
                    yield_now();
                }
                if count <= 0 {
                    while self.0.load(Ordering::Acquire) == LOCKED {
                        let mut i = 0;
                        // 休眠
                        match futex::wait(&self.0, futex::Flags::empty(), LOCKED, None)  { // 不设置超时时间
                            Ok(_) => {/*  */}
                            Err(e) => {
                                // 如果一直进入这里, 说明可能产生了死锁. 应该手动删除共享内存文件.
                                // 经过测试, 基本下不会出现死锁，出现死锁的原因大概率是共享内存文件不干净.
                                thread::sleep(Duration::from_millis(100));
                                spin_loop();
                                yield_now();
                                if i >= 500 {
                                    panic!("maybe in deadlock!, waited 1 minute."); // 1分钟后不再等待
                                }
                                i += 1;
                            }
                        }
                    }
                }
                spin_loop();
                count -= 1;
            }
        }

        fn try_lock(&self) -> bool {
            self.0
                .compare_exchange(UN_LOCKED, LOCKED, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        }

        unsafe fn unlock(&self) {
            self.0.store(UN_LOCKED, Ordering::SeqCst);
            let _ = futex::wake(&self.0, futex::Flags::empty(), u8::MAX as u32).unwrap();

        }
    }
    pub type IpcMutex<T> = lock_api::Mutex<IpcMutexRaw, T>;
    pub type IpcMutexGuard<'a, T> = lock_api::MutexGuard<'a, IpcMutexRaw, T>;
}


#[derive(Debug)]
struct Message {
    data: String,
}

fn main() -> anyhow::Result<()>{
    let rutex = mutex::Rutex::new("rustix_mmap3.bin".as_ref(), Message {
        data: "hello".to_string()
    });

    let d = Arc::new(rutex);

    let d1 = d.clone();
    let d2 = d.clone();
    let d3 = d.clone();
    let d4 = d.clone();

    thread::spawn(move || {
        loop {
            {
                let guard = d1.lock();
                // let s = size_of_val(&guard);
                // println!("val size : {:?} ", s);
            }
            thread::sleep(Duration::from_secs(2));
        }
    });
    let op = env::args().collect::<Vec<_>>().get(1).unwrap_or(&"".to_string()).clone();
    if op == "add" {
        thread::spawn(move || {
            for i in 0..10 {
                let mut d = d2.lock();
                d.data = "hello1".to_string();
            }
            println!("after add done: {:?}", d2);
            thread::sleep(Duration::from_secs(20));
        });
    } else {
        thread::spawn(move || {
            for i in 0..10 {
                let mut d = d3.lock();
                println!("message: {:?}", d.data);
            }
            println!("after sub done: {:?}", d3);
            thread::sleep(Duration::from_secs(20));
        });
    }

    // cleanup();

    thread::sleep(Duration::from_secs(200));
    println!("done");

    Ok(())
}


