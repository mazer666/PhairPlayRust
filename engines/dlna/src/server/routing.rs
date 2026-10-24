//! The routing of the control plane.
//!
//! WHY: the server answers four kinds of request — the device description,
//! the service descriptions, the SOAP actions and the GENA subscriptions —
//! and each of them has its own rules. The routing is where those rules
//! live, away from the socket handling.
//!
//! HOW: the router is a pure function of the request and the context. It
//! never touches the network, so every route can be tested without one.

use super::{DESCRIPTION_PATH, ServerContext};
use crate::description::{
    CONTROL_PATH_AV_TRANSPORT, CONTROL_PATH_CONNECTION_MANAGER, CONTROL_PATH_RENDERING_CONTROL,
    EVENT_PATH_AV_TRANSPORT, EVENT_PATH_CONNECTION_MANAGER, EVENT_PATH_RENDERING_CONTROL,
    SERVICE_AV_TRANSPORT, SERVICE_CONNECTION_MANAGER, SERVICE_RENDERING_CONTROL,
    device_description, service_description,
};
use crate::http::{HttpRequest, HttpResponse, Method};
use crate::soap::SoapRequest;
use phairplay_core::protocol::ProtocolState;

/// Answers one request.
pub(super) fn route(request: &HttpRequest, context: &ServerContext) -> HttpResponse {
    let path = request.path.as_str();
    match request.method {
        Method::Get | Method::Head => match path {
            "/" | DESCRIPTION_PATH => HttpResponse::xml(
                200,
                "OK",
                &device_description(
                    &context.friendly_name,
                    &context.udn,
                    &context.presentation_url,
                    "PhairPlayRust",
                    "PhairPlay Rust Receiver",
                ),
            ),
            "/ConnectionManager.xml" => {
                HttpResponse::xml(200, "OK", &service_description(SERVICE_CONNECTION_MANAGER))
            }
            "/AVTransport.xml" => {
                HttpResponse::xml(200, "OK", &service_description(SERVICE_AV_TRANSPORT))
            }
            "/RenderingControl.xml" => {
                HttpResponse::xml(200, "OK", &service_description(SERVICE_RENDERING_CONTROL))
            }
            _ => HttpResponse::text(404, "Not Found", "Not Found"),
        },
        Method::Post => {
            if !is_control_path(path) {
                return HttpResponse::text(404, "Not Found", "Not Found");
            }
            let soap = SoapRequest {
                control_path: path.to_string(),
                soap_action: request.header("soapaction").map(str::to_string),
                body: request.body_text(),
                peer: request.peer,
            };
            let response = context.dispatcher.dispatch(&soap);
            update_protocol_state(context);
            HttpResponse::xml(response.status, response.reason, &response.body)
        }
        Method::Subscribe => subscribe(request, context),
        Method::Unsubscribe => unsubscribe(request, context),
        Method::Other(ref method) => HttpResponse::text(
            405,
            "Method Not Allowed",
            &format!("the method {method} is not allowed"),
        ),
    }
}

/// Registers an event subscription.
fn subscribe(request: &HttpRequest, context: &ServerContext) -> HttpResponse {
    if !is_event_path(&request.path) {
        return HttpResponse::text(404, "Not Found", "Not Found");
    }
    match context
        .events
        .subscribe(request.header("callback"), &request.path)
    {
        Ok(subscription) => {
            HttpResponse::new(200, "OK", "text/plain; charset=\"utf-8\"", Vec::new())
                .with_header("SID", &subscription.sid)
                .with_header(
                    "TIMEOUT",
                    &format!("Second-{}", subscription.timeout_seconds),
                )
        }
        Err(error) => HttpResponse::text(412, "Precondition Failed", &error.to_string()),
    }
}

/// Removes an event subscription.
fn unsubscribe(request: &HttpRequest, context: &ServerContext) -> HttpResponse {
    match context.events.unsubscribe(request.header("sid")) {
        Ok(()) => HttpResponse::new(200, "OK", "text/plain; charset=\"utf-8\"", Vec::new()),
        Err(error) => HttpResponse::text(412, "Precondition Failed", &error.to_string()),
    }
}

/// Whether a path is one of the event paths.
pub(super) fn is_event_path(path: &str) -> bool {
    matches!(
        path,
        EVENT_PATH_AV_TRANSPORT | EVENT_PATH_CONNECTION_MANAGER | EVENT_PATH_RENDERING_CONTROL
    )
}

/// Whether a path is one of the control paths.
pub(super) fn is_control_path(path: &str) -> bool {
    matches!(
        path,
        CONTROL_PATH_AV_TRANSPORT
            | CONTROL_PATH_CONNECTION_MANAGER
            | CONTROL_PATH_RENDERING_CONTROL
    )
}

/// Updates the protocol state after a control action.
pub(super) fn update_protocol_state(context: &ServerContext) {
    let state = context.state.transport_state();
    let next = if state.is_active() {
        ProtocolState::Streaming
    } else {
        ProtocolState::Advertising
    };
    context
        .lifecycle
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .set_state(next);
}
