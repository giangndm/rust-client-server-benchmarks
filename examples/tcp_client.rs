use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Instant,
};

fn main() {
    let mut stream = TcpStream::connect("0.0.0.0:8080").unwrap();
    let mut buf = [0; 1 << 18];
    let mut chunk_at = Instant::now();
    let mut pkt_count = 0;
    let mut echo_len = 0;
    loop {
        stream.write_all(&buf).unwrap();
        while echo_len < buf.len() {
            echo_len += stream.read(&mut buf).unwrap();
        }
        echo_len = 0;

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
