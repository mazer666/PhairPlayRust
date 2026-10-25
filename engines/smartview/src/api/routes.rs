//! The routes of the Smart View API.
//!
//! WHAT: the handler of every path the API serves: the root of the API,
//! the resource description, the base URL, the description document, the
//! handshake of the sender and the WebSocket upgrade of the channel.
//!
//! WHY: the three steps of the discovery that Samsung documents have to end
//! somewhere: the sender reads the description document, it posts its MAC
//! address, its IP address and its device name, and it opens the channel.
//! A receiver that serves the first two but not the third is found and
//! then dropped.
//!
//! HOW: every handler is a pure function of the request and the context —
//! except the handshake, which records the peer that announced itself. The
//! answers are JSON, because the wire format of the API is not publicly
//! documented and a documented format of our own is better than a guess at
//! a proprietary one.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::api::is_websocket_upgrade;
//! use phairplay_smartview::api::ApiRequest;
//! use std::net::SocketAddr;
//!
//! let request = ApiRequest {
//!     method: phairplay_smartview::api::Method::Get,
//!     path: "/api/v2/websocket".to_string(),
//!     query: None,
//!     headers: vec![
//!         ("upgrade".to_string(), "websocket".to_string()),
//!         ("sec-websocket-version".to_string(), "13".to_string()),
//!         ("sec-websocket-key".to_string(), "dGhlIHNhbXBsZSBub25jZQ==".to_string()),
//!     ],
//!     body: Vec::new(),
//!     peer: SocketAddr::from(([127, 0, 0, 1], 40000)),
//! };
//! assert!(is_websocket_upgrade(&request));
//! ```

use serde_json::json;

use crate::api::{
    API_ROOT_PATH, ApiContext, ApiRequest, ApiResponse, BASE_URL_PATH, CHANNEL_PATH_PREFIX,
    RESOURCES_PATH, WEBSOCKET_PATH,
};
use crate::description::DESCRIPTION_PATH;
use crate::session::PeerInfo;

/// The maximum length of a text field of the handshake, in characters.
const MAX_FIELD_LENGTH: usize = 128;

/// What the routing of a request produced.
#[derive(Debug)]
pub enum RouteOutcome {
    /// An ordinary answer.
    Response(ApiResponse),
    /// The request asked for a WebSocket upgrade.
    Upgrade {
        /// The client key of the handshake.
        key: String,
        /// The session that was opened for the channel.
        session: crate::session::ActiveSession,
    },
}

/// Routes a request to the handler of its path.
#[must_use]
pub fn route(request: &ApiRequest, context: &ApiContext) -> RouteOutcome {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", DESCRIPTION_PATH) | ("HEAD", DESCRIPTION_PATH) => {
            RouteOutcome::Response(ApiResponse::xml(200, "OK", &context.description.to_xml()))
        }
        ("GET", API_ROOT_PATH) | ("HEAD", API_ROOT_PATH) => {
            RouteOutcome::Response(api_root(context))
        }
        ("GET", RESOURCES_PATH) | ("HEAD", RESOURCES_PATH) => {
            RouteOutcome::Response(resources(context))
        }
        ("GET", BASE_URL_PATH) | ("HEAD", BASE_URL_PATH) => {
            RouteOutcome::Response(base_url(context))
        }
        ("POST", API_ROOT_PATH) => RouteOutcome::Response(handshake(request, context)),
        ("GET", WEBSOCKET_PATH) => upgrade(request, context),
        ("GET", "/") | ("HEAD", "/") => RouteOutcome::Response(index(context)),
        // WHY the check comes before the arm below: the client of Samsung
        // names the channel itself, and every name under the prefix is a
        // channel of the receiver.
        _ if is_channel_path(&request.path) => upgrade(request, context),
        _ => RouteOutcome::Response(ApiResponse::not_found(&request.path)),
    }
}

/// Whether the path names a channel of the receiver.
///
/// WHY the name may not be empty: a request for `/api/v2/channels/` without
/// a name asks for a channel that does not exist, and the answer of the
/// receiver is the same as for every other unknown path.
///
/// WHY the name may not contain a slash: the name is one segment. A path
/// with a second slash is not a channel, and the receiver does not read a
/// path of its own.
#[must_use]
pub fn is_channel_path(path: &str) -> bool {
    let Some(name) = path.strip_prefix(CHANNEL_PATH_PREFIX) else {
        return false;
    };
    !name.is_empty() && !name.contains('/')
}

/// The answer to a request for the root of the API.
fn api_root(context: &ApiContext) -> ApiResponse {
    ApiResponse::json(
        200,
        "OK",
        &json!({
            "service": "urn:samsung.com:service:MultiScreenService:1",
            "version": "2",
            "baseUrl": context.base_url,
            "description": context.description_url,
            "resources": [RESOURCES_PATH, BASE_URL_PATH, WEBSOCKET_PATH],
            "device": {
                "friendlyName": context.description.friendly_name,
                "modelName": context.description.model_name,
                "udn": context.description.udn,
            },
        }),
    )
}

/// The answer to a request for the resource description.
fn resources(context: &ApiContext) -> ApiResponse {
    ApiResponse::json(
        200,
        "OK",
        &json!({
            "resources": [
                { "path": API_ROOT_PATH, "method": "GET", "description": "the root of the API" },
                { "path": RESOURCES_PATH, "method": "GET", "description": "this document" },
                { "path": BASE_URL_PATH, "method": "GET", "description": "the base URL of the receiver" },
                { "path": WEBSOCKET_PATH, "method": "GET", "description": "the channel of the session" },
                { "path": API_ROOT_PATH, "method": "POST", "description": "the handshake of the sender" },
                { "path": DESCRIPTION_PATH, "method": "GET", "description": "the description document" },
            ],
            "capabilities": {
                "video": context.description.video_codecs,
                "audio": context.description.audio_codecs,
            },
        }),
    )
}

/// The answer to a request for the base URL.
fn base_url(context: &ApiContext) -> ApiResponse {
    ApiResponse::json(
        200,
        "OK",
        &json!({
            "baseUrl": context.base_url,
            "description": context.description_url,
        }),
    )
}

/// The answer to a request for the root of the receiver.
fn index(context: &ApiContext) -> ApiResponse {
    ApiResponse::text(
        200,
        "OK",
        &format!(
            "PhairPlayRust — Samsung Smart View receiver\n\
             \n\
             Name: {}\n\
             Model: {}\n\
             Description: {}\n\
             API: {}\n",
            context.description.friendly_name,
            context.description.model_name,
            context.description_url,
            context.base_url,
        ),
    )
}

/// The handshake of the sender.
///
/// WHY: Samsung documents that the sender posts its MAC address, its IP
/// address and its device name to the device after it has read the
/// description document. The receiver keeps the three values so that the
/// session that follows can be attributed to a device.
///
/// The wire format of the post is **not** publicly documented. The receiver
/// therefore accepts a small JSON object with the three fields `mac`, `ip`
/// and `name` and ignores everything else.
fn handshake(request: &ApiRequest, context: &ApiContext) -> ApiResponse {
    let parsed: serde_json::Value = match serde_json::from_slice(&request.body) {
        Ok(value) => value,
        Err(error) => {
            return ApiResponse::json(
                400,
                "Bad Request",
                &json!({ "error": format!("the body is not a JSON object: {error}") }),
            );
        }
    };
    let peer = PeerInfo {
        mac: text_field(&parsed, "mac"),
        ip: text_field(&parsed, "ip"),
        name: text_field(&parsed, "name"),
    };
    context.sessions.announce(peer.clone());
    ApiResponse::json(
        200,
        "OK",
        &json!({
            "accepted": true,
            "device": {
                "friendlyName": context.description.friendly_name,
                "modelName": context.description.model_name,
                "udn": context.description.udn,
                "baseUrl": context.base_url,
            },
            "peer": {
                "mac": peer.mac,
                "ip": peer.ip,
                "name": peer.name,
            },
        }),
    )
}

/// Reads a text field of the handshake.
///
/// WHY: every field comes from the network. The value is truncated and
/// stripped of the control characters, so that a name of a megabyte or a
/// name with a line break cannot reach the log or the session.
fn text_field(value: &serde_json::Value, key: &str) -> Option<String> {
    let text = value.get(key)?.as_str()?;
    let cleaned: String = text
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_FIELD_LENGTH)
        .collect();
    if cleaned.trim().is_empty() {
        return None;
    }
    Some(cleaned.trim().to_string())
}

/// Accepts the WebSocket upgrade of the channel.
///
/// WHY: Smart View is a single-session protocol. A second sender that asks
/// for a channel while one is open is **rejected**, not queued.
fn upgrade(request: &ApiRequest, context: &ApiContext) -> RouteOutcome {
    if !is_websocket_upgrade(request) {
        return RouteOutcome::Response(ApiResponse::json(
            400,
            "Bad Request",
            &json!({ "error": "the request is not a WebSocket upgrade" }),
        ));
    }
    let Some(key) = request.header("sec-websocket-key") else {
        return RouteOutcome::Response(ApiResponse::json(
            400,
            "Bad Request",
            &json!({ "error": "the request carries no WebSocket key" }),
        ));
    };
    match context.sessions.open(request.peer, context.peer_limit) {
        Ok(session) => RouteOutcome::Upgrade {
            key: key.to_string(),
            session,
        },
        Err(error) => RouteOutcome::Response(ApiResponse::refused(&error.to_string())),
    }
}

/// Whether a request asks for a WebSocket upgrade.
#[must_use]
pub fn is_websocket_upgrade(request: &ApiRequest) -> bool {
    let upgrade = request
        .header("upgrade")
        .map(|value| value.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false);
    let version = request
        .header("sec-websocket-version")
        .map(|value| value.trim() == "13")
        .unwrap_or(false);
    upgrade && version && request.header("sec-websocket-key").is_some()
}

/// The answer to the WebSocket upgrade.
#[must_use]
pub fn upgrade_response(key: &str) -> ApiResponse {
    ApiResponse::new(
        101,
        "Switching Protocols",
        "text/plain; charset=\"utf-8\"",
        Vec::new(),
    )
    .with_header("Upgrade", "websocket")
    .with_header("Connection", "Upgrade")
    .with_header(
        "Sec-WebSocket-Accept",
        &crate::api::channel::accept_key(key),
    )
}
