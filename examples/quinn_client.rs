use std::{sync::Arc, time::Instant};

use quinn::{ClientConfig, Endpoint};
use quinn_proto::TransportConfig;
use quinn_utils::config_transport_config;

mod quinn_utils;

#[tokio::main]
async fn main() {
    let mut endpoint = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
    endpoint.set_default_client_config(configure_client());

    // connect to server
    let connection = endpoint
        .connect("127.0.0.1:8080".parse().unwrap(), "localhost")
        .unwrap()
        .await
        .unwrap();

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
                "{} MB/s",
                (pkt_count * buf.len()) as u64 / (1000 * elapsed.as_millis()) as u64
            );
            chunk_at = Instant::now();
            pkt_count = 0;
        }
    }
}

/// Dummy certificate verifier that treats any certificate as valid.
/// NOTE, such verification is vulnerable to MITM attacks, but convenient for testing.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

fn configure_client() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let mut config = ClientConfig::new(Arc::new(crypto));
    let mut transport_config = TransportConfig::default();
    config_transport_config(&mut transport_config);
    config.transport_config(transport_config.into());
    config
}
