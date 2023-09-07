use std::net::{SocketAddr};
use std::ptr::write;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{spawn, task};
use tokio::task::JoinHandle;
use tokio::net::{TcpSocket, TcpStream};
use tokio::net::tcp::WriteHalf;

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
            txs: Arc::new(Mutex::new(Vec::new())),
        };

        Ok(chat_server)
    }

}

pub struct ChatServer {
    bind_endpoint: SocketAddr,
    passkey: Option<String>,
    txs: Arc<Mutex<Vec<Sender<String>>>>,
}

impl ChatServer {
    pub async fn run_service(mut self)  {
        let (recv_tx, mut recv_rx) = tokio::sync::mpsc::channel::<(String, Vec<u8>)>(10); // 데이터 수신시, tx는 스트림마다 하나, rx는 핸들러에 하나 (N:1)
        let (accept_tx, mut accept_rx) = tokio::sync::mpsc::channel::<Sender<String>>(10); // 새로운 접속시 send_tx를 보낼 채널 (N:1)

        let socket = TcpSocket::new_v4().unwrap();
        socket.bind(self.bind_endpoint).unwrap();

        // 브로드캐스트 핸들러
        spawn(async move {
            let mut send_txs: Vec<Sender<String>> = Vec::new();
            loop {
                tokio::select! {
                pair = recv_rx.recv() => {
                    if let Some((nickname, msg)) = pair {
                        let to_send = String::from_utf8(msg).unwrap();
                        send_txs
                            .iter()
                            .for_each(|send_tx| {
                                let to_send_cloned = to_send.clone();
                                let send_tx_cloned = send_tx.clone();
                                let nickname_cloned = nickname.clone();
                                spawn(async move {
                                    if !send_tx_cloned.is_closed() {
                                        send_tx_cloned.send(format!("{nickname_cloned}: {to_send_cloned}")).await.unwrap_or_else(|e| eprintln!("Error! {e}"));
                                    }
                                });
                            });
                    }
                }
                send_tx = accept_rx.recv() => {
                    if let Some(send_tx) = send_tx {
                        send_txs.push(send_tx);
                    }
                }
            }
            }
        });

        // 클라이언트 연결
        let listener = socket.listen(1024).unwrap();
        while let (mut stream, _socket) = listener.accept().await.unwrap() {
            if let Some(ref key) = self.passkey {
                stream.write(&[1]).await.unwrap(); // 인증키 입력 요청
                let mut buf = Vec::new();
                buf.resize(1024, 0);
                let size = stream.read(&mut buf).await.unwrap();
                buf.resize(size, 0);
                let entered_key = String::from_utf8(buf).unwrap();
                if entered_key.as_str() != key.as_str() {
                    stream.write(&[2]).await.unwrap(); // 인증 실패
                    stream.shutdown().await.unwrap();
                    continue;
                }
            }
            stream.write(&[0]).await.unwrap(); // 접속 허가

            // 닉네임 입력 요청
            stream.write(&[100]).await.unwrap();
            let mut buf = [0u8; 64];
            let size = stream.read(&mut buf).await.unwrap();
            let nickname = String::from_utf8_lossy(&buf[..size]).to_string();

            // 데이터 송수신 핸들러 등록
            let (send_tx, mut send_rx) = tokio::sync::mpsc::channel::<String>(10);
            let recv_tx_cloned = recv_tx.clone();
            accept_tx.send(send_tx).await.unwrap();
            spawn(async move {
                let recv_tx = recv_tx_cloned;
                let nickname = nickname;
                let mut recv_buf = [0u8; 64];
                let local_addr = stream.peer_addr().unwrap(); // 클라이언트의 로컬 주소
                loop {
                    tokio::select! {
                        recv_size = stream.read(&mut recv_buf) => {
                            if let Ok(recv_size) = recv_size {
                                if recv_size == 0 {
                                    eprintln!("크기 0 수신");
                                    break;
                                }
                                recv_tx.send((nickname.clone(), recv_buf[..recv_size].to_owned())).await.unwrap();
                            } else {
                                println!("[연결 종료] {local_addr}");
                                break;
                            }
                        }
                        to_send = send_rx.recv() => {
                            if let Some(to_send) = to_send {
                                stream.write(to_send.as_bytes()).await.unwrap();
                            }
                        }
                    }
                }
            });
        }
    }
}