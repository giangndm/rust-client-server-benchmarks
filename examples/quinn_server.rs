use quinn_utils::make_server_endpoint;

mod quinn_utils;

#[tokio::main]
async fn main() {
    let (endpoint, _server_cert) = make_server_endpoint("0.0.0.0:8080".parse().unwrap()).unwrap();
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
