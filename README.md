

## [Rust_Demo 01](./demo01-rust/README.md)



### Rust Demo

- [Pin和Unpin实现自引用](./examples/rust_pin_unpin.rs)
- [PhantomData的理解](./examples/rust_pin_unpin.rs)
- [ptr裸指针操作](./examples/rust_ptr.rs)
- [各种智能指针的语义](./examples/rust_pointer.rs)
- [alloc手动分配内存](./examples/rust_alloc.rs)


### 异步 Demo

- [Once: 异步任务的初始化操作只执行一次](./examples/sync_thread.rs)
- [Condvar: 条件变量控制线程的挂起和执行](./examples/sync_thread.rs)
- [thread_local!: 为每个线程分配一个隔离的变量](./examples/sync_thread.rs)
- [Barrier: 屏障让多个线程都执行到某个点后一起往后执行](./examples/sync_thread.rs)
- [CAS: 无锁同步与优化](./examples/sync_thread.rs)
- [sync和send的demo](./examples/sync_send_sync.rs)
- [chanel通信的例子](./examples/sync_channel.rs)


### IPC Demo

- [对共享内存做CAS依然有效的例子](./examples/rustix_mmap.rs)
- [futex多进程下对线程休眠和唤醒](./examples/rustix_futex.rs)
- [共享内存实现进程锁,简单通信](./examples/rustix_mmap2.rs)

