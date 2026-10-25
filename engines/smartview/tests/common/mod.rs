//! The shared helpers of the end-to-end tests of the Smart View engine.
//!
//! WHY: the end-to-end tests need the same engine setup and the same raw
//! clients — a UDP socket for the discovery and a TCP socket for the API.
//! The helpers live here so that every test drives the receiver the same
//! way a real device would.
//!
//! HOW: the clients are asynchronous, because a blocking read would park
//! the only thread of the test runtime and the server would never get a
//! chance to answer.

// Every test target compiles this module on its own and uses only part of
// it; the unused helpers are therefore expected.
#![allow(dead_code)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use phairplay_core::capabilities::MediaCapabilities;
use phairplay_core::identity::DeviceIdentity;
use phairplay_core::media::{NullAudioSink, NullVideoSink};
use phairplay_core::protocol::{EngineContext, ProtocolEngine};
use phairplay_smartview::engine::{SmartViewConfig, SmartViewEngine};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The address the engine announces. It is only used for the announced
/// URLs, never for the socket the test connects to.
pub const ANNOUNCED_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 5);

/// A context for the engine.
pub fn context() -> EngineContext {
    let (sender, _receiver) = tokio::sync::mpsc::channel(32);
    EngineContext {
        identity: Arc::new(DeviceIdentity::with_name("Living Room")),
        capabilities: Arc::new(MediaCapabilities::empty()),
        peer_limit: phairplay_core::protocol::PeerLimit::One,
        video_sink: Arc::new(NullVideoSink),
        audio_sink: Arc::new(NullAudioSink),
        events: sender,
    }
}

/// The configuration of a test engine: free ports, loopback address.
pub fn test_config() -> SmartViewConfig {
    SmartViewConfig::default()
        .with_ports(0, 0)
        .with_local_ip(ANNOUNCED_IP)
}

/// Starts the engine and returns it together with the port of its API.
pub async fn start() -> (SmartViewEngine, u16) {
    let engine = SmartViewEngine::new(test_config());
    engine.start(context()).await.expect("the engine starts");
    let port = engine.status().tcp_port.expect("the API server has a port");
    (engine, port)
}

/// Sends a raw request and returns the whole answer.
pub async fn request(port: u16, raw: &str) -> String {
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("the connection opens");
    stream
        .write_all(raw.as_bytes())
        .await
        .expect("the request is sent");
    stream.flush().await.expect("the request is flushed");
    let mut answer = Vec::new();
    let read = tokio::time::timeout(Duration::from_secs(5), stream.read_to_end(&mut answer)).await;
    assert!(read.is_ok(), "the answer arrives");
    String::from_utf8_lossy(&answer).to_string()
}

/// The status line of an answer.
pub fn status_line(answer: &str) -> String {
    answer.lines().next().unwrap_or_default().to_string()
}

/// The body of an answer.
pub fn body(answer: &str) -> String {
    match answer.split_once("\r\n\r\n") {
        Some((_, body)) => body.to_string(),
        None => String::new(),
    }
}

/// The headers of an answer.
pub fn headers(answer: &str) -> String {
    match answer.split_once("\r\n\r\n") {
        Some((head, _)) => head.to_string(),
        None => answer.to_string(),
    }
}

/// The request that opens the WebSocket channel.
pub fn upgrade_request() -> String {
    "GET /api/v2/websocket HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
     UPGRADE: websocket\r\nCONNECTION: Upgrade\r\n\
     SEC-WEBSOCKET-KEY: dGhlIHNhbXBsZSBub25jZQ==\r\n\
     SEC-WEBSOCKET-VERSION: 13\r\n\r\n"
        .to_string()
}

/// The upgrade request of a sender that names its channel.
///
/// WHY: the client library of Samsung builds the URL of the channel from the
/// base URL of the service and the name of the channel — the remote control
/// of a television is `samsung.remote.control`.
pub fn upgrade_request_for_channel(name: &str) -> String {
    format!(
        "GET /api/v2/channels/{name} HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
         UPGRADE: websocket\r\nCONNECTION: Upgrade\r\n\
         SEC-WEBSOCKET-KEY: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         SEC-WEBSOCKET-VERSION: 13\r\n\r\n"
    )
}

/// Opens the WebSocket channel and returns the stream and the answer.
///
/// WHY: the answer of the upgrade is the head of the response — the channel
/// itself stays open, so the test may not read the stream to its end.
pub async fn open_channel(port: u16) -> (tokio::net::TcpStream, String) {
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("the connection opens");
    stream
        .write_all(upgrade_request().as_bytes())
        .await
        .expect("the upgrade is sent");
    stream.flush().await.expect("the upgrade is flushed");
    let head = read_head(&mut stream).await;
    (stream, head)
}

/// Reads the head of an answer, up to the empty line.
pub async fn read_head(stream: &mut tokio::net::TcpStream) -> String {
    let mut head: Vec<u8> = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        let read = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut byte))
            .await
            .expect("the answer arrives")
            .expect("the answer is read");
        if read == 0 {
            break;
        }
        head.push(byte[0]);
    }
    String::from_utf8_lossy(&head).to_string()
}

/// A masked WebSocket frame with the given opcode and payload.
pub fn frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0x80 | opcode];
    let mask = [0x12u8, 0x34, 0x56, 0x78];
    if payload.len() < 126 {
        out.push(0x80 | payload.len() as u8);
    } else {
        out.push(0x80 | 126);
        out.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    }
    out.extend_from_slice(&mask);
    for (index, byte) in payload.iter().enumerate() {
        out.push(byte ^ mask[index % 4]);
    }
    out
}

/// Reads one frame of the server.
///
/// WHY the payload is not unmasked: RFC 6455 says that a frame of a
/// **server** is never masked. Only the frames of the client are.
pub async fn read_frame(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
    let mut header = [0u8; 2];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut header))
        .await
        .expect("the frame arrives")
        .expect("the frame is read");
    let length = match header[1] & 0x7F {
        126 => {
            let mut extended = [0u8; 2];
            stream
                .read_exact(&mut extended)
                .await
                .expect("the length is read");
            u16::from_be_bytes(extended) as usize
        }
        127 => {
            let mut extended = [0u8; 8];
            stream
                .read_exact(&mut extended)
                .await
                .expect("the length is read");
            u64::from_be_bytes(extended) as usize
        }
        short => short as usize,
    };
    let mut payload = vec![0u8; length];
    if length > 0 {
        stream
            .read_exact(&mut payload)
            .await
            .expect("the payload is read");
    }
    payload
}

/// The address of a sender on the loopback interface.
pub fn peer_address() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 40_000))
}
