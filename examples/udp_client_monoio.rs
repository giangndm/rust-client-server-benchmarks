use std::time::Instant;

use monoio::net::udp::UdpSocket;

#[monoio::main]
async fn main() {
    let stream = UdpSocket::bind("0.0.0.0:0").unwrap();
    stream
        .connect("0.0.0.0:8080".parse().unwrap())
        .await
        .unwrap();
    let mut buf_c = Some(vec![0; 1460]);
    let mut chunk_at = Instant::now();
    let mut pkt_count = 0;

    for _ in 0..10 {
        let buf = buf_c.take().unwrap();
        let (res, buf) = stream.send(buf).await;
        buf_c.replace(buf);
    }

    loop {
        let buf = buf_c.take().unwrap();
        let (res, buf) = stream.recv(buf).await;
        let _n = res.unwrap();
        let (res, buf) = stream.send(buf).await;
        let n = res.unwrap();
        buf_c.replace(buf);

        pkt_count += 1;
        if pkt_count > 10000 {
            let elapsed = chunk_at.elapsed();
            println!(
                "{} MB/s",
                (pkt_count * n) as u64 / (1000 * elapsed.as_millis()) as u64
            );
            chunk_at = Instant::now();
            pkt_count = 0;
        }
    }
}
