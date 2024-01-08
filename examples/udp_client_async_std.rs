use std::time::Instant;

use async_std::net::UdpSocket;

#[async_std::main]
async fn main() {
    let stream = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    stream.connect("0.0.0.0:8080").await.unwrap();
    let mut buf = [0; 1460];
    let mut chunk_at = Instant::now();
    let mut pkt_count = 0;

    for _ in 0..100 {
        stream.send(&buf).await.unwrap();
    }

    loop {
        stream.recv(&mut buf).await.unwrap();
        stream.send(&buf).await.unwrap();

        pkt_count += 1;
        if pkt_count > 10000 {
            let elapsed = chunk_at.elapsed();
            println!(
                "{} MB/s",
                (pkt_count * buf.len()) as u64 / (1000 * elapsed.as_millis()) as u64
            );
            chunk_at = Instant::now();
            pkt_count = 0;
        }
    }
}
