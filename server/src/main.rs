use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let socket = tokio::net::TcpSocket::new_v4().unwrap();
    socket.bind(std::net::SocketAddr::new(std::net::IpAddr::from([127, 0, 0, 1]), 31410)).unwrap();

    let listener = socket.listen(1024).unwrap();

    let (mut stream, socket_addr) = listener.accept().await.unwrap();
    let (mut read, mut write) = stream.split();
    write.write(String::from("안녀엉").as_bytes()).await.unwrap();
}
