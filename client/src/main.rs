use std::io::{Read, Write};

fn main() {
    let mut socket = std::net::TcpStream::connect("127.0.0.1:31411").unwrap();
    let mut buf: Vec<u8> = Vec::new();
    buf.resize(1024, 0);
    let size = socket.read(&mut buf).unwrap();
    buf.resize(size, 0);

    println!("{}", String::from_utf8(buf).unwrap());
}
