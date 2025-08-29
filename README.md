

## Rust_Demo 01



### Rust Demo

- [Pin和Unpin实现自引用](./demo01-rust/examples/rust_pin_unpin.rs)
- [PhantomData的理解](./demo01-rust/examples/rust_pin_unpin.rs)
- [ptr裸指针操作](./demo01-rust/examples/rust_ptr.rs)
- [各种智能指针的语义](./demo01-rust/examples/rust_pointer.rs)
- [alloc手动分配内存](./demo01-rust/examples/rust_alloc.rs)


### 异步 Demo

- [Once: 异步任务的初始化操作只执行一次](./demo01-rust/examples/sync_thread.rs)
- [Condvar: 条件变量控制线程的挂起和执行](./demo01-rust/examples/sync_thread.rs)
- [thread_local!: 为每个线程分配一个隔离的变量](./demo01-rust/examples/sync_thread.rs)
- [Barrier: 屏障让多个线程都执行到某个点后一起往后执行](./demo01-rust/examples/sync_thread.rs)
- [CAS: 无锁同步与优化](./demo01-rust/examples/sync_thread.rs)
- [sync和send的demo](./demo01-rust/examples/sync_send_sync.rs)
- [chanel通信的例子](./demo01-rust/examples/sync_channel.rs)


### IPC Demo

- [CAS: 对共享内存做CAS依然有效的例子](./demo01-rust/examples/rustix_mmap.rs)
- [Notify: futex多进程下对线程休眠和唤醒](./demo01-rust/examples/rustix_futex.rs)
- [IPC Lock: 共享内存实现进程锁](./demo01-rust/examples/rustix_mmap2.rs)
- [signal: 使用signal实现进程退出时的清理操作。](./demo01-rust/examples/rustix_mmap2.rs)
- [event: 基于futex的event实现进程异步通知，比信号量更快相互通知。](./demo01-rust/examples/rustix_eventfd.rs)
- [socket：极简通信协议实现的socket全双工通信demo。](./demo01-rust/examples/ipc_socket.rs)
- [pipe：简单的父子进程间双向通信demo。](./demo01-rust/examples/ipc_pipe.rs)


### Other

