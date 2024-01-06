use std::{net::UdpSocket, time::Instant};

fn main() {
    let stream = UdpSocket::bind("0.0.0.0:0").unwrap();
    stream.connect("0.0.0.0:8080").unwrap();
    let mut buf = [0; 1460];
    let mut chunk_at = Instant::now();
    let mut pkt_count = 0;
    loop {
        stream.send(&buf).unwrap();
        stream.recv(&mut buf).unwrap();

        pkt_count += 3;
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
