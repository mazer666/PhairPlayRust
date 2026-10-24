//! The HTTP API of the dashboard.
//!
//! WHAT: The router and the handlers of the dashboard: the status, the
//! sessions, the counters, the configuration, the build information, the
//! two actions on a protocol and the WebSocket channel of the live events.
//!
//! WHY: The dashboard is a local control panel, not a network service. It
//! shows what the receiver does and lets the user switch a protocol on and
//! off. Every answer is read from the receiver itself; the API never
//! invents a state.
//!
//! The router serves the files of the dashboard as well, so that the
//! binary is the whole application.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_web::api::router;
//! use phairplay_web::state::AppState;
//!
//! let state = AppState::with_default_configuration()
//!     .unwrap_or_else(|error| panic!("the state must start: {error}"));
//! let app = router(state);
//! assert!(app.is::<axum::routing::get>(()));
//! ```

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use phairplay_core::build_info::BuildInfo;
use phairplay_core::config::AppConfig;
use phairplay_core::protocol::ProtocolKind;
use phairplay_ffi::error::FfiErrorCode;
use tower_http::services::ServeDir;

use crate::assets::static_root;
use crate::dto::{
    BuildInfoResponse, MetricsResponse, ProtocolActionResponse, ProtocolMetricsDto,
    ProtocolStatusDto, SessionsResponse, StatusResponse, StreamStatsDto,
};
use crate::error::ApiError;
use crate::events;
use crate::state::AppState;

/// Builds the router of the dashboard.
///
/// WHY: The files of the dashboard are served by the same binary, so that
/// the user needs one program and not two. The API keeps its own paths, and
/// everything that is not an API path is answered from the directory of the
/// dashboard.
#[must_use]
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/api/status", get(status))
        .route("/api/sessions", get(sessions))
        .route("/api/metrics", get(metrics))
        .route("/api/config", get(config).put(set_config))
        .route("/api/build-info", get(build_info))
        .route("/api/protocols/{name}/enable", post(enable_protocol))
        .route("/api/protocols/{name}/disable", post(disable_protocol))
        .route("/api/events", get(events::events));

    // WHY: the dashboard files and the API share one binary. The API keeps
    // its own paths; everything else is answered from the static directory, and
    // a path that exists in neither is a clean 404 — not a panic.
    api.with_state(state)
        .fallback_service(ServeDir::new(static_root()))
        .fallback(not_found)
}

/// The answer of a path that does not exist.
async fn not_found() -> ApiError {
    ApiError::new(
        axum::http::StatusCode::NOT_FOUND,
        FfiErrorCode::InvalidInput,
        "the requested resource does not exist",
    )
}

/// `GET /api/status` — the state of every protocol.
async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let sessions = state.sessions();
    Json(StatusResponse {
        protocols: state.protocol_statuses(),
        active_sessions: sessions.len(),
        uptime_ms: state.receiver().uptime_ms(),
    })
}

/// `GET /api/sessions` — the live sessions.
async fn sessions(State(state): State<AppState>) -> Json<SessionsResponse> {
    Json(SessionsResponse {
        sessions: state.sessions(),
    })
}

/// `GET /api/metrics` — the counters of every protocol.
async fn metrics(State(state): State<AppState>) -> Json<MetricsResponse> {
    let receiver = state.receiver();
    let protocols = ProtocolKind::ALL
        .iter()
        .map(|kind| ProtocolMetricsDto {
            protocol: kind.as_str().to_string(),
            stats: StreamStatsDto::from(&receiver.stats(*kind)),
        })
        .collect();
    Json(MetricsResponse { protocols })
}

/// `GET /api/config` — the configuration of the receiver.
async fn config(State(state): State<AppState>) -> Json<AppConfig> {
    Json(state.receiver().config())
}

/// `PUT /api/config` — replaces the configuration of the receiver.
///
/// # Errors
/// Returns [`ApiError`] when the body is not a configuration or when the
/// configuration is refused by the receiver.
async fn set_config(
    State(state): State<AppState>,
    body: Bytes,
) -> Result<Json<AppConfig>, ApiError> {
    let config: AppConfig = serde_json::from_slice(&body).map_err(|error| {
        ApiError::bad_request(format!("the body is not a configuration: {error}"))
    })?;
    state
        .receiver()
        .set_config(config)
        .map_err(ApiError::from)?;
    Ok(Json(state.receiver().config()))
}

/// `GET /api/build-info` — the identification of the build.
async fn build_info() -> Json<BuildInfoResponse> {
    Json(BuildInfoResponse::from(&BuildInfo::current()))
}

/// `POST /api/protocols/{name}/enable` — enables a protocol.
///
/// # Errors
/// Returns [`ApiError`] when the name is not a protocol or when the
/// configuration of the receiver refuses the change.
///
/// WHY: The answer carries the status of the protocol *after* the action.
/// A protocol that is enabled in the configuration but has no engine
/// reports the state `idle` and the reason in `last_error` — the dashboard
/// shows the switch as on and the reason as a warning. The request itself
/// succeeded: the configuration was changed.
async fn enable_protocol(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ProtocolActionResponse>, ApiError> {
    let protocol = state.protocol(&name)?;
    let mut config = state.receiver().config();
    config
        .protocols
        .entry(protocol.as_str().to_string())
        .or_default()
        .enabled = true;
    state
        .receiver()
        .set_config(config)
        .map_err(ApiError::from)?;

    // WHY: the failure of the engine is not the failure of the request. The
    // reason is part of the status of the protocol.
    let _ = state.receiver().start_protocol(protocol);
    let status = state.receiver().protocol_status(protocol);
    Ok(Json(ProtocolActionResponse {
        status: ProtocolStatusDto::new(&status),
    }))
}

/// `POST /api/protocols/{name}/disable` — disables a protocol.
///
/// # Errors
/// Returns [`ApiError`] when the name is not a protocol or when the
/// configuration of the receiver refuses the change.
async fn disable_protocol(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ProtocolActionResponse>, ApiError> {
    let protocol = state.protocol(&name)?;

    // WHY: the engine is stopped first, so that the socket is released
    // before the configuration says that the protocol is off.
    let _ = state.receiver().stop_protocol(protocol);
    let mut config = state.receiver().config();
    config
        .protocols
        .entry(protocol.as_str().to_string())
        .or_default()
        .enabled = false;
    state
        .receiver()
        .set_config(config)
        .map_err(ApiError::from)?;

    let status = state.receiver().protocol_status(protocol);
    Ok(Json(ProtocolActionResponse {
        status: ProtocolStatusDto::new(&status),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    use crate::test_support::Must;

    fn app() -> Router {
        let state = AppState::with_default_configuration()
            .unwrap_or_else(|error| panic!("the state must start: {error}"));
        router(state)
    }

    #[tokio::test]
    async fn the_status_is_answered() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .must("the request must be buildable"),
            )
            .await
            .must("the request must be answered");
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn an_unknown_protocol_is_a_not_found() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/protocols/telepathy/enable")
                    .method("POST")
                    .body(Body::empty())
                    .must("the request must be buildable"),
            )
            .await
            .must("the request must be answered");
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }
}
