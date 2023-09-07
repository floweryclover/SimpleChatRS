use std::io::{Read, stdin, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let mut socket = tokio::net::TcpStream::connect("127.0.0.1:31411").await.unwrap();

    // 0: 접속 허가, 1: 인증키 입력, 2: 인증 실패
    let mut buf = [0u8];
    socket.read(&mut buf).await.unwrap();
    let server_response = buf[0];

    if server_response == 0 {}
    else if server_response == 1 {
        println!("서버의 인증키를 입력하세요.");
        let mut passkey_input = String::new();
        stdin().read_line(&mut passkey_input).unwrap();
        socket.write(passkey_input.trim().as_bytes()).await.unwrap();
        let mut auth_response = [0u8];
        socket.read(&mut auth_response).await.unwrap();
        if auth_response[0] != 0 {
            eprintln!("인증에 실패하였습니다.");
            return;
        }
    } else {
        eprintln!("서버에서 알 수 없는 요청을 보내왔습니다.");
        return;
    }
    println!("서버에 연결되었습니다.");

    // 100: 닉네임 요청 필요
    let mut buf = [0u8];
    socket.read(&mut buf).await.unwrap();
    let server_response = buf[0];
    if server_response == 100 {
        println!("닉네임을 입력해 주세요.");
        let mut nickname_input = String::new();
        stdin().read_line(&mut nickname_input).unwrap();
        socket.write(nickname_input.trim().as_bytes()).await.unwrap();
    } else {
        eprintln!("서버에서 알 수 없는 요청을 보내왔습니다.");
        return;
    }

    let (send_tx, mut send_rx) = tokio::sync::mpsc::channel::<String>(10);
    tokio::spawn(async move {
        let mut recv_buf = [0u8; 64];
        loop {
            tokio::select! {
                to_send = send_rx.recv() => {
                    if let Some(to_send) = to_send {
                        socket.write(to_send.as_bytes()).await.unwrap();
                    }
                }
                recv = socket.read(&mut recv_buf) => {
                    if let Ok(recv) = recv {
                        println!("{}", String::from_utf8_lossy(&recv_buf[..recv]));
                    } else {
                        eprintln!("[연결 종료]");
                        break;
                    }
                }
            }
        }
    });

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        input.pop().unwrap();
        send_tx.send(input).await.unwrap();
    }
}
