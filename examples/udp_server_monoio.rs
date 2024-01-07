use monoio::net::udp::UdpSocket;

#[monoio::main]
async fn main() {
    let listener = UdpSocket::bind("0.0.0.0:8080").unwrap();
    let mut buf_c: Option<Vec<u8>> = Some(vec![0; 1460]);
    loop {
        let buf = buf_c.take().expect("");
        let (res, buf) = listener.recv_from(buf).await;
        let (_n, from) = res.unwrap();
        let (_res, buf2) = listener.send_to(buf, from).await;
        buf_c.replace(buf2);
    }
}
