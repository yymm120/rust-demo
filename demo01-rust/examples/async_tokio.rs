#![allow(unused)]
use std::io::{self, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use std::{
    io::{Read, Write},
    net::TcpStream,
};


fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:34254")?;
    stream.write(&[1])?;
    stream.read(&mut [0; 128]);
    Ok(())
}

#[cfg(test)]
mod tcpstream {
    use std::io::{self, Result};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
    use std::time::Duration;
    use std::{
        io::{Read, Write},
        net::TcpStream,
    };
    #[test]
    fn connect() {
        let addrs = [
            SocketAddr::from(([127, 0, 0, 1], 8081)),
            SocketAddr::from(([127, 0, 0, 1], 8082)),
        ];
        if let Ok(stream) = TcpStream::connect(&addrs[..]) {
            println!("Connected to Server!");
        } else {
            println!("Couldn't connect to server");
        }
    } // closed stream

    #[test]
    fn connect_timeout() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 3333));
        let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(1))
            .expect("Couldn't connect to the server");
    }

    #[test]
    fn peer_addr() {
        let stream =
            TcpStream::connect("127.0.0.1:8083").expect("Couldn't connect to the server...");
        assert_eq!(
            stream.peer_addr().unwrap(),
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8083))
        )
    }

    #[test]
    fn local_addr() {
        let stream =
            TcpStream::connect("127.0.0.1:8083").expect("Couldn't connect to the server...");
        assert_eq!(
            stream.local_addr().unwrap().ip(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        );
    }

    #[test]
    fn shutdown() {
        let stream =
            TcpStream::connect("127.0.0.1:8083").expect("Couldn't connect to the server...");
        stream
            .shutdown(std::net::Shutdown::Both)
            .expect("shutdown call failed");
    }

    #[test]
    fn try_clone() {
        let stream =
            TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect to the server...");
        let stream_clone = stream.try_clone().expect("clone failed...");
    }

    #[test]
    fn set_read_timeout() {
        let stream =
            TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect to the server...");
        stream
            .set_read_timeout(None)
            .expect("set_read_timeout call failed");

        let result = stream.set_read_timeout(Some(Duration::new(0, 0)));
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn set_write_timeou() {
        let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let result = stream.set_write_timeout(Some(Duration::new(0, 0)));
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    }

    #[test]
    fn read_timeout() {
        let stream =
            TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect to the server...");
        stream
            .set_read_timeout(None)
            .expect("set_read_timeout call failed");
        assert_eq!(stream.read_timeout().unwrap(), None);
    }

    #[test]
    fn write_timeout() {
        let stream =
            TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect to the server...");
        stream
            .set_write_timeout(None)
            .expect("set_write_timeout call failed");
        assert_eq!(stream.write_timeout().unwrap(), None);
    }

    /// 在套接字上从其连接的远程地址接收数据，而不从队列中删除该数据。成功时，返回查看的字节数。
    /// 连续调用返回相同的数据。这是通过将 MSG_PEEK 作为标志传递给底层 recv 系统调用来完成的。
    #[test]
    fn peek() {
        let stream =
            TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect to the server...");
        let mut buf = [0; 10];
        let len = stream.peek(&mut buf).expect("peek failed");
    }

    /// 设置此套接字上 TCP_NODELAY 选项的值。
    /// 如果设置，此选项将禁用 Nagle 算法。这意味着即使只有少量数据，数据段也总是尽快发送。当不设置时，数据会被缓冲，直到有足够的量发送出去，从而避免频繁发送小数据包
    #[test]
    fn set_no_delay() {
        let stream =
            TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect to the server...");
        stream.set_nodelay(true).expect("set_nodelay call failed");
        assert_eq!(stream.nodelay().unwrap_or(false), true);
    }

    /// 设置此套接字上的 IP_TTL 选项的值。

    /// 该值设置从此套接字发送的每个数据包中使用的生存时间字段。
    #[test]
    fn set_ttl() {
        let stream =
            TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect to the server...");
        stream.set_ttl(100).expect("set_ttl call failed");
        assert_eq!(stream.ttl().unwrap_or(0), 100);
    }

    /// 获取此套接字上 SO_ERROR 选项的值。

    /// 这将检索底层套接字中存储的错误，并清除进程中的字段。这对于检查调用之间的错误很有用。
    #[test]
    fn take_error() {
        let stream =
            TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect to the server...");
        stream.take_error().expect("No error was expected...");
    }

    /// 将此 TCP 流移入或移出非阻塞模式。
    /// 这将导致读、写、接收和发送操作变得非阻塞，即立即从它们的调用中返回。如果IO操作成功，则返回Ok，无需执行任何操作。如果 IO 操作无法完成并需要重试，则会返回类型为 io::ErrorKind::WouldBlock 的错误。
    /// 在Unix平台上，调用该方法相当于调用fcntl FIONBIO。在 Windows 上调用此方法对应于调用 ioctlsocket FIONBIO。
    #[test]
    fn set_non_blocking() {
        let mut stream =
            TcpStream::connect("127.0.0.1:7878").expect("Couldn't connect to the server...");
        stream
            .set_nonblocking(true)
            .expect("set_nonblocking call failed");

        let mut buf = vec![];
        loop {
            match stream.read_to_end(&mut buf) {
                Ok(_) => break,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // 等待网络套接字准备就绪，通常通过特定于平台的 API（例如 epoll 或 IOCP）实现
                    wait_for_fd();
                }
                Err(e) => panic!("encountered IO error: {e}"),
            };
        }
        println!("bytes: {buf:?}");
    }
}

#[cfg(test)]
mod impl_write_for_tcpstream {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::{BufWriter, IoSlice};

    /// write: 将buf写入buffer. 返回写入的字节数
    #[test]
    fn write() -> std::io::Result<()> {
        let mut buffer = File::create("foo.txt")?;

        let len = buffer.write(b"some bytes")?;

        Ok(())
    }
    /// flush: 刷新此输出流，确保所有中间缓冲的内容到达目的地。
    #[test]
    fn flush() -> anyhow::Result<()>  {
        let mut buffer = BufWriter::new(File::create("foo.txt")?);
        buffer.write_all(b"some bytes");

        buffer.flush()?;

        Ok(())
    }
    fn write_vectored() -> anyhow::Result<()>  {
        let data1 = [1; 8];
        let data2 = [15; 8];
        let io_slice1 = IoSlice::new(&data1);
        let io_slice2 = IoSlice::new(&data2);

        let mut buffer = File::create("foo.txt")?;
        buffer.write_vectored(&[io_slice1, io_slice2])?;

        Ok(())
    }
    fn is_write_vectored() -> anyhow::Result<()>  {Ok(())}
    fn write_all() -> anyhow::Result<()>  {
        let mut buffer = File::create("foo.txt")?;

        buffer.write_all(b"some bytes")?;

        Ok(())
    }
    fn write_all_vectored() {}
    #[test]
    fn write_fmt() -> anyhow::Result<()> {
        let mut buffer = File::create("foo.txt")?;

        // 应该更倾向于使用`write!()`
        write!(buffer, "{:.*}", 2, 1.234567)?;
        // 尽管`write_fmt()`与上一行代码等价
        buffer.write_fmt(format_args!("{:.*}", 2, 1.234567))?;
        Ok(())
    }
    
    /// by_ref 返回一个引用。这个引用仍然借用write，所以可以调用write的方法。
    #[test]
    fn by_ref() -> anyhow::Result<()>  {
        let mut buffer = File::create("foo.txt")?;

        let reference = buffer.by_ref();

        // 我们可以像使用原始缓冲区一样使用 Reference
        reference.write_all(b"some bytes")?;
    }
}
