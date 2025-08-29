#![allow(unused)]

use std::{future::Future, time::Duration};

use executor::Executor;
#[cfg(test)]
mod future_test {
    use std::future::Future;
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};
    use std::str::from_utf8;
    use std::thread;
    use std::time::Duration;

    fn handle_client(mut stream: TcpStream) {
        let mut data = [0 as u8; 50]; // 50 byte buffer
        while match stream.read(&mut data) {
            Ok(size) => {
                stream.write(&data[0..size]).unwrap();
                true
            }
            Err(_) => {
                println!(
                    "An error occured, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } {}
    }

    #[test]
    fn socket_create_tcp_listener() {
        println!("==");
        let socket_path = "127.0.0.1:3333";
        let listener = TcpListener::bind(socket_path).unwrap();

        let server_thread = thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        println!("New connection: {}", stream.peer_addr().unwrap());
                        thread::spawn(move || {
                            handle_client(stream);
                        });
                    }
                    Err(e) => {
                        println!("Failed to establish a connection: {}", e);
                    }
                }
            }
        });

        thread::sleep(Duration::from_millis(2));
        let client_thread = thread::spawn(move || {
            match TcpStream::connect(socket_path) {
                Ok(mut stream) => {
                    let msg = b"Hello!";
                    stream.write(msg).unwrap();
                    let mut data = [0 as u8; 6]; // using 6 byte buffer
                    match stream.read_exact(&mut data) {
                        Ok(_) => {
                            if &data == msg {
                                println!("Reply is ok!");
                            } else {
                                let text = from_utf8(&data).unwrap();
                                println!("Unexpectd reply {}", text);
                            }
                        }
                        Err(e) => {
                            println!("Failed to receive data: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to connect: {}", e)
                }
            }
        });
        server_thread.join().unwrap();
        client_thread.join().unwrap();
    }

    #[test]
    fn future_base() {}
}

// pub mod runtime {
//     use std::{collections::HashMap, sync::Arc, task::Waker};

//     pub struct Task {

//     }
//     pub struct Reactor {

//     }
//     pub struct Executor {
//         task_map: HashMap<usize, Arc<Task>>,
//     }
//     impl Executor {
//         pub fn new() -> Executor {
//             Self {
//                 task_map: HashMap::new(),
//             }
//         }
//         pub fn run(&mut self) {
//             for task in self.task_map.iter() {
//                 let waker = Waker::from(Arc::clone(task));
//                 waker.wake();
//             }
//         }
//     }

// }

pub mod timer_future {
    use std::{
        future::Future,
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
        thread,
        time::Duration,
    };

    use futures::FutureExt;

    pub struct TimeFuture {
        shared_state: Arc<Mutex<SharedState>>,
    }

    pub struct SharedState {
        completed: bool,
        waker: Option<Waker>,
    }

    impl Future for TimeFuture {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut shared_state = self.shared_state.lock().unwrap();
            if shared_state.completed {
                Poll::Ready(())
            } else {
                shared_state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
    impl TimeFuture {
        pub fn new(duration: Duration) -> Self {
            let shared_state = Arc::new(Mutex::new(SharedState {
                completed: false,
                waker: None,
            }));
            let thread_shared_state = shared_state.clone();
            thread::spawn(move || {
                thread::sleep(duration);
                let mut shared_state = thread_shared_state.lock().unwrap();
                shared_state.completed = true;
                if let Some(waker) = shared_state.waker.take() {
                    waker.wake()
                }
            });
            TimeFuture { shared_state }
        }
    }
}

pub mod executor {
    use crate::timer_future::TimeFuture;
    use futures::{
        future::{BoxFuture, FutureExt},
        task::{waker_ref, ArcWake, UnsafeFutureObj},
    };
    use std::{
        borrow::BorrowMut, clone, collections::VecDeque, future::{Future, IntoFuture}, process::Output, sync::{
            mpsc::{sync_channel, Receiver, SyncSender},
            Arc, Mutex,
        }, task::Context, time::Duration
    };

    pub struct Executor {
        has_pending: bool,
        task_list: Vec<Box<dyn Future<Output = ()> + 'static>>,
    }

    impl Executor {
        pub fn new() -> Self {
            Executor::with_capacity(8)
        }
        pub fn with_capacity(num: usize) -> Executor {
            Executor {
                task_list: Vec::with_capacity(num),
                has_pending: false,
            }
        }
        pub fn task(&mut self, task: Box<dyn Future<Output = ()>>) -> &Executor {
            self.task_list.push(task);
            self.has_pending = true;
            self
        }
        pub fn run(&mut self) {
            let a = self.task_list.get(0).unwrap();
            let b = async {
                ()
            };
            // let is_pending =b.boxed().as_mut().poll(cx).is_pending();
        }
    }
}
fn main() {
    let mut executor = Executor::new();
    executor.task(Box::new(async {
        println!("hello1");
    }));
    // spanwer.spawn(async {
    //     println!("hello1");
    //     timer_future::TimeFuture::new(Duration::new(2, 0)).await;
    //     println!("done");
    // });
    // drop(spanwer);
    // executor.run();
}
