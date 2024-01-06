use std::net::UdpSocket;

fn main() {
    let listener = UdpSocket::bind("0.0.0.0:8080").unwrap();
    let mut buf = [0; 1460];
    loop {
        let (n, from) = listener.recv_from(&mut buf).unwrap();
        listener.send_to(&buf[..n], from).unwrap();
    }
}
