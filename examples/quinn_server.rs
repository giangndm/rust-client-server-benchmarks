use std::net::SocketAddr;
use clap::Parser;
use quinn::Endpoint;

#[derive(Parser, Debug, Clone)]
#[clap(name = "server")]
pub struct ServerOpt {
    /// Listen addr
    #[clap(long, default_value = "0.0.0.0:8080")]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() {
    let opt = ServerOpt::parse();

    let endpoint = Endpoint::server(quinn_plaintext::server_config(), opt.listen).unwrap();
    while let Some(incoming_conn) = endpoint.accept().await {
        let conn = incoming_conn.await.unwrap();
        println!("new connection from {}", conn.remote_address());
        tokio::spawn(async move {
            let (mut send, mut recv) = conn.accept_bi().await.unwrap();
            let mut buf = [0; 1 << 18];
            loop {
                if let Some(n) = recv.read(&mut buf).await.unwrap() {
                    if n == 0 {
                        println!("received 0, done");
                        break;
                    }
                    send.write_all(&buf[..n]).await.unwrap();
                } else {
                    break;
                }
            }
            println!("connection closed");
        });
    }
}
