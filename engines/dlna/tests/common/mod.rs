//! The shared helpers of the end-to-end tests of the DLNA control plane.
//!
//! WHY: the two test files need the same engine setup and the same raw
//! HTTP client. The helpers live here so that both files test the same
//! thing in the same way.
//!
//! HOW: the client is asynchronous, because a blocking read would park
//! the only thread of the test runtime and the server would never get a
//! chance to answer.

// Each test target compiles this module on its own and uses only part of
// it; the unused helpers are therefore expected.
#![allow(dead_code)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use phairplay_core::capabilities::MediaCapabilities;
use phairplay_core::identity::DeviceIdentity;
use phairplay_core::media::{NullAudioSink, NullVideoSink};
use phairplay_core::protocol::{EngineContext, ProtocolEngine};
use phairplay_dlna::engine::{DlnaConfig, DlnaEngine};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The address the engine announces. It is only used for the announced URL
/// and for the GENA guard, never for the socket the test connects to.
pub const ANNOUNCED_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 5);

/// A context for the engine.
pub fn context() -> EngineContext {
    let (sender, _receiver) = tokio::sync::mpsc::channel(32);
    EngineContext {
        identity: Arc::new(DeviceIdentity::with_name("Living Room")),
        capabilities: Arc::new(MediaCapabilities::empty()),
        video_sink: Arc::new(NullVideoSink),
        audio_sink: Arc::new(NullAudioSink),
        events: sender,
    }
}

/// Starts the engine and returns it together with its port.
pub async fn start() -> (DlnaEngine, u16) {
    let engine = DlnaEngine::new(
        DlnaConfig::default()
            .with_ports(0, 0)
            .with_local_ip(ANNOUNCED_IP),
    );
    engine.start(context()).await.expect("the engine starts");
    let port = engine
        .status()
        .tcp_port
        .expect("the description server has a port");
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

/// Sends a raw request from a given source address and returns the answer.
///
/// WHY: the "one peer at a time" rule can only be proven over the wire
/// with two *different* peers. The loopback range 127.0.0.0/8 is entirely
/// local, so a second source address is available without a second
/// machine.
pub async fn request_from(source: &str, port: u16, raw: &str) -> String {
    let socket = tokio::net::TcpSocket::new_v4().expect("the socket exists");
    let source: std::net::SocketAddr = format!("{source}:0")
        .parse()
        .expect("the source address is valid");
    socket.bind(source).expect("the source address is bound");
    let target = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = socket.connect(target).await.expect("the connection opens");
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

/// The body of a SOAP request.
pub fn soap_body(action: &str, arguments: &str) -> String {
    format!(
        "<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\"><s:Body>\
         <u:{action} xmlns:u=\"urn:schemas-upnp-org:service:AVTransport:1\">{arguments}</u:{action}>\
         </s:Body></s:Envelope>",
        action = action,
        arguments = arguments
    )
}

/// A SOAP request with the given action and arguments.
pub fn soap(action: &str, arguments: &str) -> String {
    let body = soap_body(action, arguments);
    format!(
        "POST /upnp/control/AVTransport HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
         SOAPACTION: \"urn:schemas-upnp-org:service:AVTransport:1#{action}\"\r\n\
         CONTENT-TYPE: text/xml; charset=\"utf-8\"\r\nCONTENT-LENGTH: {length}\r\n\r\n{body}",
        action = action,
        length = body.len(),
        body = body
    )
}

/// A SOAP request with a body that is sent verbatim.
pub fn soap_raw(body: &str) -> String {
    format!(
        "POST /upnp/control/AVTransport HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
         CONTENT-TYPE: text/xml; charset=\"utf-8\"\r\nCONTENT-LENGTH: {length}\r\n\r\n{body}",
        length = body.len(),
        body = body
    )
}
