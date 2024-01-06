// Copyright (c) 2023 The TQUIC Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cell::RefCell;
use std::cmp;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use bytes::Bytes;
use clap::Parser;
use log::debug;
use log::error;
use tquic::Config;
use tquic::Connection;
use tquic::Endpoint;
use tquic::Error;
use tquic::PacketInfo;
use tquic::TlsConfig;
use tquic::TransportHandler;
use tquic::TIMER_GRANULARITY;

mod tquic_native_utils;

use tquic_native_utils::QuicSocket;
use tquic_native_utils::Result;

const MAX_BUF_SIZE: usize = 65536;

#[derive(Parser, Debug, Clone)]
#[clap(name = "client")]
pub struct ClientOpt {
    /// Log level, support OFF/ERROR/WARN/INFO/DEBUG/TRACE.
    #[clap(long, default_value = "INFO", value_name = "STR")]
    pub log_level: log::LevelFilter,

    /// Override server's address.
    #[clap(short, long, value_name = "ADDR")]
    pub connect_to: SocketAddr,

    /// Connection idle timeout in microseconds.
    #[clap(long, default_value = "5000", value_name = "TIME")]
    pub idle_timeout: u64,

    /// File used for session resumption.
    #[clap(long, value_name = "FILE")]
    pub session_file: Option<String>,

    /// Save TLS key log into the given file.
    #[clap(long, value_name = "FILE")]
    pub keylog_file: Option<String>,

    /// Save QUIC qlog into the given file.
    #[clap(long, value_name = "FILE")]
    pub qlog_file: Option<String>,
}

// A simple http/0.9 client over QUIC.
struct Client {
    /// QUIC endpoint.
    endpoint: Endpoint,

    /// Socket connecting to server.
    sock: Rc<QuicSocket>,

    /// Client context.
    context: Rc<RefCell<ClientContext>>,

    /// Packet read buffer.
    recv_buf: Vec<u8>,
}

impl Client {
    fn new(option: &ClientOpt) -> Result<Self> {
        let mut config = Config::new()?;
        config.set_max_idle_timeout(option.idle_timeout);
        config.set_send_udp_payload_size(1460);
        config.set_recv_udp_payload_size(1460);

        let tls_config = TlsConfig::new_client_config(vec![b"http/0.9".to_vec()], false)?;
        config.set_tls_config(tls_config);

        let context = Rc::new(RefCell::new(ClientContext { finish: false }));
        let handlers = ClientHandler::new(option, context.clone());

        let sock = Rc::new(QuicSocket::new_client_socket(option.connect_to.is_ipv4())?);

        Ok(Client {
            endpoint: Endpoint::new(Box::new(config), false, Box::new(handlers), sock.clone()),
            sock,
            context,
            recv_buf: vec![0u8; MAX_BUF_SIZE],
        })
    }

    fn finish(&self) -> bool {
        let context = self.context.borrow();
        context.finish()
    }

    fn process_read_event(&mut self) -> Result<()> {
        if self.context.borrow().finish() {
            return Ok(());
        }
        let timeout = cmp::max(self.endpoint.timeout(), Some(TIMER_GRANULARITY));

        self.sock.socket.set_read_timeout(timeout)?;

        // Read datagram from the socket.
        let (len, local, remote) = match self.sock.recv_from(&mut self.recv_buf) {
            Ok(v) => v,
            Err(e) => {
                self.endpoint.on_timeout(Instant::now());
                return Ok(());
            }
        };
        debug!("socket recv recv {} bytes from {:?}", len, remote);

        let pkt_buf = &mut self.recv_buf[..len];
        let pkt_info = PacketInfo {
            src: remote,
            dst: local,
            time: Instant::now(),
        };

        // Process the incoming packet.
        match self.endpoint.recv(pkt_buf, &pkt_info) {
            Ok(_) => {}
            Err(e) => {
                error!("recv failed: {:?}", e);
            }
        };

        Ok(())
    }
}

struct ClientContext {
    finish: bool,
}

impl ClientContext {
    fn set_finish(&mut self, finish: bool) {
        self.finish = finish
    }

    fn finish(&self) -> bool {
        self.finish
    }
}

struct ClientHandler {
    session_file: Option<String>,
    keylog_file: Option<String>,
    qlog_file: Option<String>,
    context: Rc<RefCell<ClientContext>>,
    stats_at: Instant,
    stats_len: usize,
    /// Waiting write
    queue: VecDeque<Bytes>,
}

impl ClientHandler {
    fn new(option: &ClientOpt, context: Rc<RefCell<ClientContext>>) -> Self {
        Self {
            session_file: option.session_file.clone(),
            keylog_file: option.keylog_file.clone(),
            qlog_file: option.qlog_file.clone(),
            context,
            stats_at: Instant::now(),
            stats_len: 0,
            queue: VecDeque::new(),
        }
    }
}

impl TransportHandler for ClientHandler {
    fn on_conn_created(&mut self, conn: &mut Connection) {
        debug!("{} connection is created", conn.trace_id());

        if let Some(session_file) = &self.session_file {
            if let Ok(session) = std::fs::read(session_file) {
                if conn.set_session(&session).is_err() {
                    error!("{} session resumption failed", conn.trace_id());
                }
            }
        }

        if let Some(keylog_file) = &self.keylog_file {
            if let Ok(file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(keylog_file)
            {
                conn.set_keylog(Box::new(file));
            } else {
                error!("{} set key log failed", conn.trace_id());
            }
        }

        if let Some(qlog_file) = &self.qlog_file {
            if let Ok(qlog) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(qlog_file)
            {
                conn.set_qlog(
                    Box::new(qlog),
                    "client qlog".into(),
                    format!("id={}", conn.trace_id()),
                );
            } else {
                error!("{} set qlog failed", conn.trace_id());
            }
        }
    }

    fn on_conn_established(&mut self, conn: &mut Connection) {
        debug!("{} connection is established", conn.trace_id());

        match conn.stream_write(0, vec![0; MAX_BUF_SIZE].into(), false) {
            Ok(write) => {
                assert_eq!(write, MAX_BUF_SIZE);
            }
            Err(Error::Done) => {}
            Err(e) => {
                error!("stream send failed {:?}", e);
            }
        };
    }

    fn on_conn_closed(&mut self, conn: &mut Connection) {
        debug!("{} connection is closed", conn.trace_id());
        let mut context = self.context.try_borrow_mut().unwrap();
        context.set_finish(true);
        if let Some(session_file) = &self.session_file {
            if let Some(session) = conn.session() {
                std::fs::write(session_file, session).ok();
            }
        }
    }

    fn on_stream_created(&mut self, conn: &mut Connection, stream_id: u64) {
        debug!("{} stream {} is created", conn.trace_id(), stream_id);
    }

    fn on_stream_readable(&mut self, conn: &mut Connection, stream_id: u64) {
        loop {
            let mut buf = vec![0; 1460];
            if let Ok((read, fin)) = conn.stream_read(stream_id, &mut buf) {
                debug!(
                    "{} read {} bytes from stream {}, fin: {}",
                    conn.trace_id(),
                    read,
                    stream_id,
                    fin
                );
                self.stats_len += read;
                if self.stats_len > 100_000_000 {
                    let elapsed = self.stats_at.elapsed();
                    println!(
                        "{} MB/s",
                        (self.stats_len) as u64 / (1000 * elapsed.as_millis()) as u64
                    );
                    self.stats_at = Instant::now();
                    self.stats_len = 0;
                }

                let mut buf = Bytes::from(buf);
                buf.truncate(read);

                match conn.stream_write(stream_id, buf.clone(), fin) {
                    Ok(write) => {
                        if write < read {
                            debug!(
                                "{} stream {} write only {} bytes put remain to queue",
                                conn.trace_id(),
                                stream_id,
                                write
                            );
                            self.queue.push_back(buf.slice(write..));
                            conn.stream_want_write(stream_id, true);
                        } else {
                            debug!(
                                "{} stream {} write all {} bytes",
                                conn.trace_id(),
                                stream_id,
                                write
                            );
                        }
                    }
                    Err(Error::Done) => {}
                    Err(e) => {
                        error!("stream send failed {:?}", e);
                    }
                };
            } else {
                break;
            }
        }
    }

    fn on_stream_writable(&mut self, conn: &mut Connection, stream_id: u64) {
        debug!("{} stream {} is writable", conn.trace_id(), stream_id,);
        conn.stream_want_write(stream_id, false);
        match self.queue.front_mut() {
            Some(buf) => {
                let pre_size = buf.len();
                match conn.stream_write(stream_id, buf.clone(), false) {
                    Ok(written_size) => {
                        if written_size < pre_size {
                            buf.split_to(written_size);
                            return;
                        } else {
                            self.queue.pop_front();
                        }
                    }
                    Err(Error::Done) => {
                        return;
                    }
                    Err(e) => {
                        error!("stream send failed {:?}", e);
                        return;
                    }
                };
            }
            None => return,
        };
    }

    fn on_stream_closed(&mut self, conn: &mut Connection, stream_id: u64) {
        debug!("{} stream {} is closed", conn.trace_id(), stream_id,);
    }

    fn on_new_token(&mut self, _conn: &mut Connection, _token: Vec<u8>) {}
}

pub const TIMER_GRANULARITY2: Duration = Duration::from_millis(10);

fn main() -> Result<()> {
    let option = ClientOpt::parse();

    // Initialize logging.
    env_logger::builder().init();

    // Create client.
    let mut client = Client::new(&option)?;

    // Connect to server.
    client.endpoint.connect(
        client.sock.local_addr(),
        option.connect_to,
        None,
        None,
        None,
    )?;

    // Run event loop
    loop {
        // Process connections.
        client.endpoint.process_connections()?;
        if client.finish() {
            break;
        }

        // Process timeout events
        client.process_read_event()?;
    }
    Ok(())
}
