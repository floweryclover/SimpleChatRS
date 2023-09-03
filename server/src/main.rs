#[tokio::main]
async fn main() {

    let mut input_buf = String::new();
    let chat_server = ChatServerBuilder::new()
        .with_endpoint([127,0,0,1], 31411)
        .with_passkey_auth(String::from("passkey"));

    let _ = chat_server.run_service().await;
}
