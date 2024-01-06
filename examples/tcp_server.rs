use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    loop {
        let (mut stream, _) = listener.accept().unwrap();
        std::thread::spawn(move || {
            let mut buf = [0; 1 << 18];
            loop {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    println!("received 0, done");
                    return;
                }
                stream.write_all(&buf[..n]).unwrap();
            }
        });
    }
}
