//! The SOAP dispatcher of the UPnP control plane.
//!
//! WHY: a control point drives the renderer with SOAP actions. The dispatcher
//! is the place where an untrusted request becomes a state change, so it is
//! where the rules live: which action is allowed, what its arguments are
//! worth, and what a second control point gets when the renderer is already
//! busy.
//!
//! HOW: the body is parsed with the safe reader of [`crate::xml`], which
//! rejects a document type and every entity that XML does not define. The
//! action is taken from the `SOAPACTION` header and cross-checked against
//! the body, so a request cannot claim one action and carry another.
//!
//! The answer of a fault carries the UPnP error code that the specification
//! defines, because the control point decides what to show the user from
//! that code.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_dlna::soap::{SoapDispatcher, SoapRequest};
//! use phairplay_dlna::state::RendererState;
//! use std::net::SocketAddr;
//! use std::sync::Arc;
//!
//! let state = Arc::new(RendererState::new());
//! let dispatcher = SoapDispatcher::new(state, Arc::new(phairplay_dlna::soap::NoEvents));
//! let peer: SocketAddr = "192.168.1.20:50000".parse().expect("an address");
//!
//! let response = dispatcher.dispatch(&SoapRequest {
//!     control_path: "/upnp/control/ConnectionManager".to_string(),
//!     soap_action: Some("urn:schemas-upnp-org:service:ConnectionManager:1#GetProtocolInfo".to_string()),
//!     body: "<s:Envelope xmlns:s=\"urn:x\"><s:Body><u:GetProtocolInfo xmlns:u=\"urn:y\"/></s:Body></s:Envelope>".to_string(),
//!     peer,
//! });
//! assert_eq!(response.status, 200);
//! assert!(response.body.contains("http-get:*:video/mp4:*"));
//! ```

use std::net::SocketAddr;
use std::sync::Arc;

use crate::description::{
    CONTROL_PATH_AV_TRANSPORT, CONTROL_PATH_CONNECTION_MANAGER, CONTROL_PATH_RENDERING_CONTROL,
};
use crate::state::RendererState;

mod services;
mod values;

pub use values::{format_duration, parse_time};

use values::collect_arguments;

use crate::xml::{XmlDocument, XmlError};

/// The protocol information the renderer announces as a sink.
pub const SINK_PROTOCOL_INFO: &str =
    "http-get:*:video/mp4:*,http-get:*:audio/mpeg:*,http-get:*:image/jpeg:*,http-get:*:image/png:*";

/// The UPnP error codes the dispatcher reports.
pub mod error_code {
    /// The action is not known.
    pub const INVALID_ACTION: u32 = 401;
    /// An argument is missing or malformed.
    pub const INVALID_ARGS: u32 = 402;
    /// The action failed.
    pub const ACTION_FAILED: u32 = 501;
    /// The transition is not available in the current state.
    pub const TRANSITION_NOT_AVAILABLE: u32 = 701;
    /// The seek mode is not supported.
    pub const SEEK_MODE_NOT_SUPPORTED: u32 = 710;
    /// The seek target is out of range.
    pub const ILLEGAL_SEEK_TARGET: u32 = 711;
}

/// Receives the state changes of the renderer.
///
/// WHY: the dispatcher must not know how the events are delivered. The
/// trait keeps the control plane independent of the event plane and makes
/// both testable on their own.
pub trait EventSink: Send + Sync + std::fmt::Debug {
    /// Publishes a state change of a service.
    fn publish(&self, service_path: &str, last_change: &str);
}

/// An event sink that drops every event.
#[derive(Debug, Default)]
pub struct NoEvents;

impl EventSink for NoEvents {
    fn publish(&self, _service_path: &str, _last_change: &str) {}
}

/// An event sink that forwards to several sinks.
///
/// WHY: the control plane has two consumers for a state change — the GENA
/// subscribers on the network and the session registry of the core. The
/// composite keeps the dispatcher independent of both.
#[derive(Debug, Default)]
pub struct CompositeEventSink {
    sinks: Vec<Arc<dyn EventSink>>,
}

impl CompositeEventSink {
    /// A sink that forwards to the given sinks.
    #[must_use]
    pub fn new(sinks: Vec<Arc<dyn EventSink>>) -> Self {
        Self { sinks }
    }
}

impl EventSink for CompositeEventSink {
    fn publish(&self, service_path: &str, last_change: &str) {
        for sink in &self.sinks {
            sink.publish(service_path, last_change);
        }
    }
}

/// One SOAP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoapRequest {
    /// The control URL the request was sent to.
    pub control_path: String,
    /// The `SOAPACTION` header, if the control point sent one.
    pub soap_action: Option<String>,
    /// The request body.
    pub body: String,
    /// The address of the control point.
    pub peer: SocketAddr,
}

/// The answer to a SOAP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoapResponse {
    /// The HTTP status code.
    pub status: u16,
    /// The HTTP reason phrase.
    pub reason: &'static str,
    /// The response body.
    pub body: String,
}

/// The SOAP dispatcher.
#[derive(Debug)]
pub struct SoapDispatcher {
    state: Arc<RendererState>,
    events: Arc<dyn EventSink>,
}

impl SoapDispatcher {
    /// A dispatcher that works on the given state.
    #[must_use]
    pub fn new(state: Arc<RendererState>, events: Arc<dyn EventSink>) -> Self {
        Self { state, events }
    }

    /// The state the dispatcher works on.
    #[must_use]
    pub fn state(&self) -> &Arc<RendererState> {
        &self.state
    }

    /// Handles one request.
    ///
    /// # Errors
    /// Never — every failure is reported as a SOAP fault, because a control
    /// point expects a fault and not a dropped connection.
    #[must_use]
    pub fn dispatch(&self, request: &SoapRequest) -> SoapResponse {
        let document = match XmlDocument::parse(&request.body) {
            Ok(document) => document,
            Err(error) => return bad_request(&error.to_string()),
        };
        let body = document
            .root()
            .child("Body")
            .or_else(|| document.root().child("Body"));
        let Some(body) = body else {
            return bad_request("the SOAP body has no Body element");
        };
        let Some(action_element) = body.children().next() else {
            return bad_request("the SOAP body has no action");
        };
        let body_action = action_element.local_name().to_string();

        let action = match &request.soap_action {
            Some(header) => match parse_soap_action(header) {
                Some(name) => {
                    if name != body_action {
                        return bad_request(&format!(
                            "the SOAPACTION header names `{name}` but the body carries `{body_action}`"
                        ));
                    }
                    name
                }
                None => return bad_request("the SOAPACTION header is malformed"),
            },
            None => body_action.clone(),
        };

        let arguments = collect_arguments(action_element);
        match request.control_path.as_str() {
            CONTROL_PATH_CONNECTION_MANAGER => self.connection_manager(&action),
            CONTROL_PATH_AV_TRANSPORT => self.av_transport(&action, &arguments, request.peer),
            CONTROL_PATH_RENDERING_CONTROL => self.rendering_control(&action, &arguments),
            _ => not_found(&format!("unknown control path {}", request.control_path)),
        }
    }
}

/// Extracts the action name of a `SOAPACTION` header.
///
/// WHY: the header carries the action in two shapes — the bare name and the
/// service URI with a `#` in front of the name. Both are accepted, nothing
/// else is.
#[must_use]
pub fn parse_soap_action(header: &str) -> Option<String> {
    let trimmed = header.trim().trim_matches('"').trim();
    if trimmed.is_empty() {
        return None;
    }
    let name = match trimmed.split_once('#') {
        Some((_, name)) => name,
        None => trimmed,
    };
    let name = name.trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    Some(name.to_string())
}

/// Collects the direct child elements of the action element.
/// Builds a successful answer.
pub(super) fn ok(action: &str, service: &str, values: &[(&str, &str)]) -> SoapResponse {
    let mut inner = String::new();
    for (key, value) in values {
        inner.push_str(&format!("<{key}>{}</{key}>", escape_xml(value)));
    }
    let body = envelope(&format!(
        "<u:{action}Response xmlns:u=\"urn:schemas-upnp-org:service:{service}:1\">{inner}</u:{action}Response>"
    ));
    SoapResponse {
        status: 200,
        reason: "OK",
        body,
    }
}

/// Builds a fault answer.
pub(super) fn fault(
    status: u16,
    reason: &'static str,
    code: u32,
    description: &str,
) -> SoapResponse {
    let body = envelope(&format!(
        "<s:Fault><faultcode>s:Client</faultcode><faultstring>UPnPError</faultstring><detail>\
         <UPnPError xmlns=\"urn:schemas-upnp-org:control-1-0\"><errorCode>{code}</errorCode>\
         <errorDescription>{}</errorDescription></UPnPError></detail></s:Fault>",
        escape_xml(description)
    ));
    SoapResponse {
        status,
        reason,
        body,
    }
}

/// Builds the answer to a request that could not be understood.
fn bad_request(reason: &str) -> SoapResponse {
    fault(400, "Bad Request", error_code::INVALID_ACTION, reason)
}

/// Builds the answer to a request for an unknown control path.
fn not_found(reason: &str) -> SoapResponse {
    fault(404, "Not Found", error_code::INVALID_ACTION, reason)
}

/// Builds the answer to the request of a second control point.
pub(super) fn session_rejected(error: &crate::state::SessionError) -> SoapResponse {
    fault(
        503,
        "Service Unavailable",
        error_code::TRANSITION_NOT_AVAILABLE,
        &error.to_string(),
    )
}

/// Builds the answer to an unknown action.
pub(super) fn invalid_action(action: &str) -> SoapResponse {
    fault(
        500,
        "Internal Server Error",
        error_code::INVALID_ACTION,
        &format!("invalid action: {action}"),
    )
}

/// Wraps a payload in a SOAP envelope.
fn envelope(content: &str) -> String {
    format!(
        "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
         s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body>{content}</s:Body></s:Envelope>"
    )
}

/// Escapes a text for use in an XML document.
fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

/// The reasons an XML document was rejected.
impl From<XmlError> for SoapResponse {
    fn from(error: XmlError) -> Self {
        bad_request(&error.to_string())
    }
}

#[cfg(test)]
mod tests;
