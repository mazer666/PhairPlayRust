//! The tests of the HTTP API.
//!
//! WHY: the API is the surface a Galaxy device talks to. A wrong answer
//! means that the device is found and then dropped, so every route is
//! checked against the answer it has to produce.

#![allow(clippy::expect_used, clippy::unwrap_used)]
use std::net::SocketAddr;
use std::sync::Arc;

use super::*;
use crate::description::DeviceDescription;
use crate::session::SessionRegistry;

/// The address of a sender on the loopback interface.
fn peer() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 40_000))
}

/// A context of the API.
fn context() -> Arc<ApiContext> {
    let (sender, _receiver) = tokio::sync::mpsc::channel(8);
    Arc::new(ApiContext {
        description: DeviceDescription::new(
            "Living Room",
            "PhairPlay Rust Receiver",
            "uuid:abc",
            "http://127.0.0.1:8001/api/v2/",
        ),
        base_url: "http://127.0.0.1:8001/api/v2/".to_string(),
        description_url: "http://127.0.0.1:8001/description.xml".to_string(),
        sessions: Arc::new(SessionRegistry::new()),
        events: sender,
        limits: ApiLimits::default(),
    })
}

/// A request without a body.
fn request(method: &str, path: &str) -> ApiRequest {
    ApiRequest {
        method: Method::parse(method),
        path: path.to_string(),
        query: None,
        headers: Vec::new(),
        body: Vec::new(),
        peer: peer(),
    }
}

/// The body of an answer.
fn body(response: &ApiResponse) -> String {
    String::from_utf8_lossy(&response.body).to_string()
}

/// The answer of a route that is an ordinary response.
fn response_of(outcome: RouteOutcome) -> ApiResponse {
    match outcome {
        RouteOutcome::Response(response) => response,
        RouteOutcome::Upgrade { .. } => panic!("the route answered with an upgrade"),
    }
}

#[test]
fn the_api_root_names_the_service_and_the_base_url() {
    let context = context();
    let response = response_of(route(&request("GET", API_ROOT_PATH), &context));
    assert_eq!(response.status, 200);
    let text = body(&response);
    assert!(
        text.contains("urn:samsung.com:service:MultiScreenService:1"),
        "{text}"
    );
    assert!(text.contains("http://127.0.0.1:8001/api/v2/"), "{text}");
    assert!(text.contains("Living Room"), "{text}");
}

#[test]
fn the_resource_description_lists_the_paths() {
    let context = context();
    let response = response_of(route(&request("GET", RESOURCES_PATH), &context));
    assert_eq!(response.status, 200);
    let text = body(&response);
    assert!(text.contains(WEBSOCKET_PATH), "{text}");
    assert!(text.contains(BASE_URL_PATH), "{text}");
}

#[test]
fn the_base_url_is_answered() {
    let context = context();
    let response = response_of(route(&request("GET", BASE_URL_PATH), &context));
    assert_eq!(response.status, 200);
    assert!(body(&response).contains("http://127.0.0.1:8001/api/v2/"));
}

#[test]
fn the_description_document_is_served_as_xml() {
    let context = context();
    let response = response_of(route(&request("GET", "/description.xml"), &context));
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "text/xml; charset=\"utf-8\"");
    assert!(body(&response).contains("<friendlyName>Living Room</friendlyName>"));
}

#[test]
fn an_unknown_path_is_a_not_found() {
    let context = context();
    let response = response_of(route(&request("GET", "/api/v2/nonsense"), &context));
    assert_eq!(response.status, 404);
    assert!(body(&response).contains("unknown path"));
}

#[test]
fn a_post_that_is_not_a_handshake_is_refused() {
    let context = context();
    let response = response_of(route(&request("POST", BASE_URL_PATH), &context));
    assert_eq!(response.status, 404);
}

#[test]
fn the_handshake_records_the_announced_peer() {
    // WHY: the sender posts its MAC address, its IP address and its device
    // name — Samsung documents the three values.
    let context = context();
    let mut request = request("POST", API_ROOT_PATH);
    request.body = serde_json::json!({
        "mac": "aabbccddeeff",
        "ip": "192.168.1.20",
        "name": "Galaxy S23",
    })
    .to_string()
    .into_bytes();
    let response = response_of(route(&request, &context));
    assert_eq!(response.status, 200);
    let announced = context.sessions.announced().expect("the peer was recorded");
    assert_eq!(announced.mac.as_deref(), Some("aabbccddeeff"));
    assert_eq!(announced.ip.as_deref(), Some("192.168.1.20"));
    assert_eq!(announced.name.as_deref(), Some("Galaxy S23"));
}

#[test]
fn the_handshake_refuses_a_body_that_is_not_json() {
    let context = context();
    let mut request = request("POST", API_ROOT_PATH);
    request.body = b"not json at all".to_vec();
    let response = response_of(route(&request, &context));
    assert_eq!(response.status, 400);
    assert!(context.sessions.announced().is_none());
}

#[test]
fn a_control_character_in_the_name_of_the_sender_is_removed() {
    // WHY: the name reaches the log and the session. A line break in it
    // would forge a log line.
    let context = context();
    let mut request = request("POST", API_ROOT_PATH);
    request.body = serde_json::json!({ "name": "Galaxy\u{0}\u{1b}S23" })
        .to_string()
        .into_bytes();
    let response = response_of(route(&request, &context));
    assert_eq!(response.status, 200);
    let announced = context.sessions.announced().expect("the peer was recorded");
    assert_eq!(announced.name.as_deref(), Some("GalaxyS23"));
}

#[test]
fn an_over_long_name_of_the_sender_is_truncated() {
    let context = context();
    let mut request = request("POST", API_ROOT_PATH);
    request.body = serde_json::json!({ "name": "x".repeat(4096) })
        .to_string()
        .into_bytes();
    let response = response_of(route(&request, &context));
    assert_eq!(response.status, 200);
    let announced = context.sessions.announced().expect("the peer was recorded");
    assert_eq!(announced.name.map(|name| name.chars().count()), Some(128));
}

#[test]
fn a_handshake_without_a_name_is_accepted() {
    let context = context();
    let mut request = request("POST", API_ROOT_PATH);
    request.body = serde_json::json!({ "mac": "aabbccddeeff" })
        .to_string()
        .into_bytes();
    let response = response_of(route(&request, &context));
    assert_eq!(response.status, 200);
    let announced = context.sessions.announced().expect("the peer was recorded");
    assert!(announced.name.is_none());
}

/// A request that asks for a WebSocket upgrade.
fn upgrade_request() -> ApiRequest {
    let mut request = request("GET", WEBSOCKET_PATH);
    request.headers = vec![
        ("upgrade".to_string(), "websocket".to_string()),
        ("connection".to_string(), "Upgrade".to_string()),
        ("sec-websocket-version".to_string(), "13".to_string()),
        (
            "sec-websocket-key".to_string(),
            "dGhlIHNhbXBsZSBub25jZQ==".to_string(),
        ),
    ];
    request
}

#[test]
fn the_channel_of_a_sender_is_accepted_under_its_own_name() {
    // WHY: the client library of Samsung builds the URL of the channel from
    // the base URL of the service and the name of the channel — the remote
    // control of a television is `samsung.remote.control`. A receiver that
    // only answers at one fixed path is not found by a sender that names
    // its channel.
    let context = context();
    let mut request = request("GET", "/api/v2/channels/samsung.remote.control");
    request.headers = vec![
        ("upgrade".to_string(), "websocket".to_string()),
        ("connection".to_string(), "Upgrade".to_string()),
        ("sec-websocket-version".to_string(), "13".to_string()),
        (
            "sec-websocket-key".to_string(),
            "dGhlIHNhbXBsZSBub25jZQ==".to_string(),
        ),
    ];
    match route(&request, &context) {
        RouteOutcome::Upgrade { key, .. } => {
            assert_eq!(key, "dGhlIHNhbXBsZSBub25jZQ==");
        }
        RouteOutcome::Response(response) => panic!(
            "the channel of the sender was refused: {}",
            response.status
        ),
    }
}

#[test]
fn a_path_without_a_channel_name_is_no_channel() {
    // WHY: a request for `/api/v2/channels/` without a name asks for a
    // channel that does not exist, and a name with a second slash is not a
    // channel at all.
    assert!(!is_channel_path("/api/v2/channels/"));
    assert!(!is_channel_path("/api/v2/channels/a/b"));
    assert!(!is_channel_path("/api/v2/websocket"));
    assert!(is_channel_path("/api/v2/channels/samsung.remote.control"));
    assert!(is_channel_path("/api/v2/channels/a"));
}

#[test]
fn the_websocket_upgrade_is_accepted() {
    let context = context();
    match route(&upgrade_request(), &context) {
        RouteOutcome::Upgrade { key, session } => {
            assert_eq!(key, "dGhlIHNhbXBsZSBub25jZQ==");
            assert!(context.sessions.is_active());
            assert_eq!(session.peer.name, None);
        }
        RouteOutcome::Response(response) => panic!("the upgrade was refused: {}", response.status),
    }
}

#[test]
fn a_second_channel_is_refused_and_the_first_one_keeps_the_slot() {
    // WHY: Smart View serves one sender at a time. The second sender is
    // rejected, not queued.
    let context = context();
    let _first = match route(&upgrade_request(), &context) {
        RouteOutcome::Upgrade { session, .. } => session,
        RouteOutcome::Response(_) => panic!("the first upgrade was refused"),
    };
    let response = response_of(route(&upgrade_request(), &context));
    assert_eq!(response.status, 409);
    assert!(body(&response).contains("already open"));
}

#[test]
fn a_request_that_is_not_an_upgrade_is_refused() {
    let context = context();
    let response = response_of(route(&request("GET", WEBSOCKET_PATH), &context));
    assert_eq!(response.status, 400);
    assert!(body(&response).contains("not a WebSocket upgrade"));
}

#[test]
fn an_upgrade_without_a_key_is_refused() {
    let context = context();
    let mut request = request("GET", WEBSOCKET_PATH);
    request.headers = vec![
        ("upgrade".to_string(), "websocket".to_string()),
        ("sec-websocket-version".to_string(), "13".to_string()),
    ];
    let response = response_of(route(&request, &context));
    assert_eq!(response.status, 400);
}

#[test]
fn the_answer_of_the_upgrade_carries_the_accept_key() {
    let response = upgrade_response("dGhlIHNhbXBsZSBub25jZQ==");
    assert_eq!(response.status, 101);
    let accept = response
        .extra_headers
        .iter()
        .find(|(name, _)| name == "Sec-WebSocket-Accept")
        .map(|(_, value)| value.clone());
    assert_eq!(accept.as_deref(), Some("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
}

#[test]
fn the_accept_key_is_the_one_of_rfc_6455() {
    // WHY: the example of RFC 6455, section 1.3 — the sender checks this
    // value, and a wrong one makes it drop the connection.
    assert_eq!(
        accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
        "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
    );
}

#[test]
fn the_method_is_parsed_and_spelled_back() {
    assert_eq!(Method::parse("GET"), Method::Get);
    assert_eq!(Method::parse("HEAD"), Method::Head);
    assert_eq!(Method::parse("POST"), Method::Post);
    assert_eq!(Method::parse("DELETE"), Method::Other("DELETE".to_string()));
    assert_eq!(Method::Get.as_str(), "GET");
    assert_eq!(Method::Other("DELETE".to_string()).as_str(), "DELETE");
}

#[test]
fn the_limits_are_the_documented_ones() {
    let limits = ApiLimits::default();
    assert_eq!(limits.max_header_count, 64);
    assert_eq!(limits.max_header_bytes, 8 * 1024);
    assert_eq!(limits.max_body_bytes, 64 * 1024);
    assert_eq!(limits.max_frame_bytes, 1024 * 1024);
}
