use std::{time::Instant, net::SocketAddr};
use clap::Parser;
use quinn::Endpoint;

#[derive(Parser, Debug, Clone)]
#[clap(name = "client")]
pub struct ClientOpt {
    /// Listen addr
    #[clap(long)]
    servers: Vec<SocketAddr>,
}


#[tokio::main]
async fn main() {
    let mut endpoint = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
    endpoint.set_default_client_config(quinn_plaintext::client_config());

    for server in ClientOpt::parse().servers {
        // connect to 2server
        let connection = endpoint
            .connect(server, "localhost")
            .unwrap()
            .await
            .unwrap();

        tokio::spawn(async move {
            println!("[client] connected: addr={}", connection.remote_address());
            let (mut send, mut recv) = connection.open_bi().await.unwrap();

            let mut buf = [0; 1 << 18];
            let mut chunk_at = Instant::now();
            let mut pkt_count = 0;
            let mut echo_len = 0;
            loop {
                send.write_all(&buf).await.unwrap();
                while echo_len < buf.len() {
                    echo_len += recv.read(&mut buf).await.unwrap().unwrap();
                }
                assert_eq!(echo_len, buf.len());
                echo_len = 0;

                pkt_count += 1;
                if pkt_count > 1000 {
                    let elapsed = chunk_at.elapsed();
                    println!(
                        "{} {} MB/s",
                        server, (pkt_count * buf.len()) as u64 / (1000 * elapsed.as_millis()) as u64
                    );
                    chunk_at = Instant::now();
                    pkt_count = 0;
                }
            }
        });
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}