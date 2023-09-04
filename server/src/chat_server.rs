use std::net::{SocketAddr};
use tokio::io::AsyncWriteExt;
use tokio::spawn;
use tokio::task::JoinHandle;
use tokio::net::{TcpSocket};

pub struct ChatServerBuilder {
    bind_endpoint: Option<SocketAddr>,
    passkey: Option<String>,
}

impl ChatServerBuilder {
    pub fn new() -> Self {
        ChatServerBuilder {
            bind_endpoint: None,
            passkey: None,
        }
    }

    pub fn with_endpoint(mut self, endpoint: SocketAddr) -> Self {
        self.bind_endpoint = Some(endpoint);
        self
    }

    pub fn with_passkey_auth(mut self, passkey: String) -> Self {
        self.passkey = Some(passkey);
        self
    }

    pub fn build(self) -> Result<ChatServer, String> {
        if self.bind_endpoint == None {
            return Err(String::from("Please set bind endpoint."))
        }

        let chat_server = ChatServer {
            bind_endpoint: self.bind_endpoint.unwrap(),
            passkey: self.passkey,
        };

        Ok(chat_server)
    }

}

pub struct ChatServer {
    bind_endpoint: SocketAddr,
    passkey: Option<String>,
}

impl ChatServer {
    pub async fn run_service(self)  {
        let socket = TcpSocket::new_v4().unwrap();
        socket.bind(self.bind_endpoint).unwrap();

        let listener = socket.listen(1024).unwrap();
        // let mut connections = Vec::new();
        while let (mut stream, socket) = listener.accept().await.unwrap() {
            let (read, mut write) = stream.split();
            write.write(String::from("Hello").as_bytes()).await.unwrap();
        }
    }
}