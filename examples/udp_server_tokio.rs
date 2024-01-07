use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let listener = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
    let mut buf = [0; 1460];
    loop {
        let (n, from) = listener.recv_from(&mut buf).await.unwrap();
        listener.send_to(&buf[..n], from).await.unwrap();
    }
}
