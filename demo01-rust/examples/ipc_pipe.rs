use std::{env, thread};
use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

const HEAD_SIZE: usize = 8;

/// 有两种情况会导致死锁
/// 1. 其中一个进程关闭, 而另一个进程不知道它关闭, 此时将陷入无限循环中.
/// 2. 如果不满足读取条件, 将永远无法读取, 此时会永久的阻塞.
/// 
/// PIPE管道读取, 可用性很低, 尽管可以, 但一般不怎么用于通信, 主要用途是获取子进程的执行结果。
fn main() {
    let op = env::args().collect::<Vec<_>>().get(1).unwrap_or(&"both".to_string()).clone();

    if op == "send" {
        thread::spawn(|| {
            let mut sin = std::io::stdin();
            loop {
                let mut head = [0u8; HEAD_SIZE];
                if let Ok(_) = sin.read_exact(&mut head) {
                    let len = u64::from_be_bytes(head) as usize;
                    let mut data = vec![0u8; len];
                    if let Ok(_) = sin.read_exact(&mut data) {
                        let len: [u8; 8] = 1u64.to_be_bytes();
                        std::io::stdout().write_all(&len).unwrap();
                        std::io::stdout().write_all(&data).unwrap();
                        std::io::stdout().flush().unwrap();

                    }
                }
            }
        });
        loop {
            let len: [u8; 8] = 1u64.to_be_bytes();
            let data: [u8; 1] = [1];
            std::io::stdout().write_all(&len).unwrap();
            std::io::stdout().write_all(&data).unwrap();
            std::io::stdout().flush().unwrap();
            sleep(Duration::from_millis(2000));
        }

    } else {

        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg("ipc_pipe")
            .arg("send")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn cat.");

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        let mut stdout = child.stdout.take().expect("Failed to open stdout");

        let mut reader = BufReader::new(stdout);


        thread::spawn(move || {
            let len: [u8; 8] = 1u64.to_be_bytes();
            let data: [u8; 1] = [2];
            stdin.write_all(&len).unwrap();
            stdin.write_all(&data).unwrap();
            stdin.flush().unwrap();
        });


        loop {
            let mut head = [0u8; HEAD_SIZE];
            if let Ok(_) = reader.read_exact(&mut head) {
                let len = u64::from_be_bytes(head) as usize;
                println!("{}", len);

                let mut data = vec![0u8; len];
                if let Ok(_) = reader.read_exact(&mut data) {
                    println!("{:#?}", data);
                } else {
                    break
                }
            } else {
                break
            }
            println!("..........");
        }
    }
}
