use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use std::time::Duration;
use bincode;
use bincode::{config, Decode, Encode};

// 定义消息枚举
#[derive(Encode, Decode, Debug, Clone)]
enum Message {
    Text(String),
    Data(Vec<u8>),
    Command { cmd: String, args: Vec<String> },
    Ping,
    Pong,
}


fn send_message(stream: &mut UnixStream, message: Message) -> std::io::Result<()> {
    let config = config::standard();
    let res = bincode::encode_to_vec(&message, config).unwrap();
    println!("{:#?}",res.len());

    // 先发送长度（4字节）
    stream.write_all(&res.len().to_be_bytes())?;
    // 再发送数据
    stream.write_all(&res)?;
    stream.flush()?;

    Ok(())
}

fn receive_message(stream: &mut UnixStream) -> anyhow::Result<Message> {
    let mut str = [0u8; 8];
    stream.read_exact(&mut str)?;
    let len = u64::from_be_bytes(str) as usize;
    println!("{:#?}", len);

    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;
    let config = config::standard();
    let decoded: (Message, usize) = bincode::decode_from_slice(&buffer, config).unwrap();
    println!("{:#?}", decoded);
    todo!()
}

fn run_server(socket_path: &str) -> anyhow::Result<()> {
    // 清理可能存在的旧套接字文件
    let _ = std::fs::remove_file(socket_path);

    // 创建Unix域套接字监听器
    let listener = UnixListener::bind(socket_path)?;
    println!("🚀 服务器启动，监听在: {}", socket_path);

    // 接受客户端连接
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("✅ 客户端连接成功");

                // 为每个客户端创建新线程
                let socket_path = socket_path.to_string();
                thread::spawn(move || {
                    if let Err(e) = handle_client(&mut stream) {
                        eprintln!("客户端处理错误: {}", e);
                    }
                });
            }
            Err(err) => {
                eprintln!("❌ 接受连接错误: {}", err);
                break;
            }
        }
    }

    // 清理套接字文件
    std::fs::remove_file(socket_path)?;
    Ok(())
}
fn run_client(socket_path: &str) -> anyhow::Result<()> {
    // 连接服务器
    let mut stream = UnixStream::connect(socket_path)?;
    println!("✅ 连接到服务器: {}", socket_path);

    // 发送各种类型的消息
    let messages = vec![
        Message::Text("BBBBBBBBBBB".to_string()),
        Message::Data(vec![1, 2, 3, 4, 5]),
        Message::Command {
            cmd: "ls".to_string(),
            args: vec!["-la".to_string()],
        },
        Message::Ping,
    ];

    for message in messages {
        println!("📤 发送: {:?}", message);
        match send_message(&mut stream, message.clone()) {
            Ok(_) => {}
            Err(e) => {eprintln!("send error!")}
        }

        // 接收响应
        match receive_message(&mut stream) {
            Ok(response) => {
                println!("📥 收到响应: {:?}", response);
            }
            Err(e) => {
                eprintln!("❌ 接收响应错误: {}", e);
                break;
            }
        }

        thread::sleep(Duration::from_millis(500));
    }

    println!("🎉 客户端完成");
    Ok(())
}

fn handle_client(stream: &mut UnixStream) -> anyhow::Result<()> {
    loop {
        // 接收消息
        let message = match receive_message(stream) {
            Ok(msg) => msg,
            Err(e) => {
                println!("客户端断开连接: {}", e);
                break;
            }
        };

        println!("📨 收到消息: {:?}", message);

        // 根据消息类型处理
        match message {
            Message::Text(text) => {
                let response = Message::Text(format!("ECHO: {}", text));
                send_message(stream, response)?;
            }
            Message::Data(data) => {
                let response = Message::Data(data.iter().map(|b| b.wrapping_add(1)).collect());
                send_message(stream, response)?;
            }
            Message::Command { cmd, args } => {
                let response = Message::Text(format!("执行命令: {} {:?}", cmd, args));
                send_message(stream, response)?;
            }
            Message::Ping => {
                send_message(stream, Message::Pong)?;
            }
            Message::Pong => {
                println!("收到Pong响应");
            }
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {

    let socket_path = "/mydata/code/rust-demo/target/demo01.sock";

    let args: Vec<String> = std::env::args().collect();
    //
    if args.len() > 1 && args[1] == "client" {
        run_client(socket_path)?
    } else {
        run_server(socket_path)?
    }

    run_server(socket_path)?;

    Ok(())
}