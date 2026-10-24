
use super::*;

use crate::error::AirPlayError;
use tokio::io::AsyncWriteExt;

fn limits() -> AirPlayLimits {
    AirPlayLimits::default()
}

async fn read_all(input: &[u8]) -> Result<Option<RtspRequest>, AirPlayError> {
    let (client, server) = tokio::io::duplex(64 * 1024);
    let payload = input.to_vec();
    tokio::spawn(async move {
        let mut client = client;
        let _ = client.write_all(&payload).await;
    });
    let mut buffer = StreamBuffer::new(server);
    let reader = RtspRequestReader::new(limits());
    reader.read_request(&mut buffer).await
}

#[tokio::test]
async fn a_simple_request_is_read() {
    let request = read_all(b"OPTIONS rtsp://x/y RTSP/1.0\r\nCSeq: 3\r\n\r\n")
        .await
        .expect("read")
        .expect("request");
    assert_eq!(request.method, RtspMethod::Options);
    assert_eq!(request.cseq(), Some(3));
}

#[tokio::test]
async fn a_body_is_read_exactly() {
    let request = read_all(b"POST /play HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello")
        .await
        .expect("read")
        .expect("request");
    assert_eq!(request.body, b"hello");
}

#[tokio::test]
async fn an_end_of_stream_yields_no_request() {
    let request = read_all(b"").await.expect("read");
    assert!(request.is_none());
}

#[tokio::test]
async fn a_declared_body_longer_than_the_limit_is_rejected() {
    let error = read_all(b"POST /play HTTP/1.1\r\nContent-Length: 99999999\r\n\r\n")
        .await
        .expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }), "{error}");
}

#[tokio::test]
async fn a_malformed_content_length_is_rejected() {
    let error = read_all(b"POST /play HTTP/1.1\r\nContent-Length: abc\r\n\r\n")
        .await
        .expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }), "{error}");
}

#[tokio::test]
async fn a_negative_content_length_is_rejected() {
    let error = read_all(b"POST /play HTTP/1.1\r\nContent-Length: -1\r\n\r\n")
        .await
        .expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }), "{error}");
}

#[tokio::test]
async fn too_many_headers_are_rejected() {
    let mut input = Vec::new();
    input.extend_from_slice(b"OPTIONS rtsp://x/y RTSP/1.0\r\n");
    for index in 0..200 {
        input.extend_from_slice(format!("X-Test-{index}: value\r\n").as_bytes());
    }
    input.extend_from_slice(b"\r\n");
    let error = read_all(&input).await.expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }), "{error}");
}

#[tokio::test]
async fn an_overlong_header_line_is_rejected() {
    let mut input = Vec::new();
    input.extend_from_slice(b"OPTIONS rtsp://x/y RTSP/1.0\r\n");
    input.extend_from_slice(b"X-Long: ");
    input.extend(std::iter::repeat_n(b'a', 9000));
    input.extend_from_slice(b"\r\n\r\n");
    let error = read_all(&input).await.expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }), "{error}");
}

#[tokio::test]
async fn an_overlong_request_line_is_rejected() {
    let mut input = Vec::new();
    input.extend_from_slice(b"OPTIONS ");
    input.extend(std::iter::repeat_n(b'a', 20_000));
    input.extend_from_slice(b" RTSP/1.0\r\n\r\n");
    let error = read_all(&input).await.expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }), "{error}");
}

#[tokio::test]
async fn a_header_without_a_colon_is_rejected() {
    let error = read_all(b"OPTIONS rtsp://x/y RTSP/1.0\r\nbroken\r\n\r\n")
        .await
        .expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }), "{error}");
}

#[tokio::test]
async fn a_photo_body_may_use_the_photo_limit() {
    let mut config = phairplay_core::config::AppConfig::default();
    config.limits.max_photo_bytes = 4096;
    config
        .protocols
        .insert("airplay".to_string(), phairplay_core::config::ProtocolConfig {
            enabled: true,
            port: 7000,
            require_pin: false,
            max_message_bytes: 1024,
        });
    let limits = AirPlayLimits::from_config(&config);
    let (client, server) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let mut client = client;
        let _ = client.write_all(b"PUT /photo HTTP/1.1\r\nContent-Length: 2048\r\n\r\n").await;
        let _ = client.write_all(&vec![0u8; 2048]).await;
    });
    let mut buffer = StreamBuffer::new(server);
    let reader = RtspRequestReader::new(limits);
    let request = reader
        .read_request(&mut buffer)
        .await
        .expect("read")
        .expect("request");
    assert_eq!(request.body.len(), 2048);
}

#[tokio::test]
async fn a_stalled_peer_hits_the_deadline() {
    let (client, server) = tokio::io::duplex(1024);
    let mut client = client;
    tokio::spawn(async move {
        let _ = client.write_all(b"OPTIONS rtsp://x/y RTSP/1.0\r\n").await;
        // Never finish the request.
        tokio::time::sleep(Duration::from_secs(30)).await;
    });
    let mut limits = limits();
    limits.idle_timeout = Duration::from_millis(80);
    let mut buffer = StreamBuffer::new(server);
    let reader = RtspRequestReader::new(limits);
    let error = reader.read_request(&mut buffer).await.expect_err("must fail");
    assert!(matches!(error, AirPlayError::Timeout { .. }), "{error}");
}
