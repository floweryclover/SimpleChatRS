use crate::chat_server::{ChatServerBuilder, ChatServer};

mod chat_server;

#[tokio::main]
async fn main() {

    let mut input_buf = String::new();
    let chat_server = ChatServerBuilder::new()
        .with_endpoint(std::net::SocketAddr::new(std::net::IpAddr::from([127, 0, 0, 1]), 31411))
        .with_passkey_auth(String::from("passkey"))
        .build().unwrap();

    let _ = chat_server.run_service().await;
}
