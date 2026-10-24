//! The failures of the HTTP API.
//!
//! WHAT: The error type of every endpoint. It carries the HTTP status, the
//! stable numeric code of the core and the explanation.
//!
//! WHY: The dashboard has to tell three failures apart: "you asked for
//! something that does not exist" (404), "what you asked for is wrong"
//! (400) and "the receiver is broken" (500). A single opaque 500 would
//! make every failure look the same, and the user would see a spinner
//! instead of an answer.
//!
//! The numeric code is the same one that the UniFFI bridge uses, so that
//! the dashboard and the native shells report a failure the same way.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_web::error::ApiError;
//!
//! let error = ApiError::unknown_protocol("telepathy");
//! assert_eq!(error.status.as_u16(), 404);
//! assert_eq!(error.code, 1);
//! ```

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use phairplay_ffi::error::FfiError;
use phairplay_ffi::error::FfiErrorCode;

use crate::dto::{ErrorBody, ErrorResponse};

/// A failure of the API.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    /// The HTTP status of the answer.
    pub status: StatusCode,
    /// The stable numeric code of the failure.
    pub code: i32,
    /// The human-readable explanation of the failure.
    pub message: String,
}

impl ApiError {
    /// Builds a failure.
    #[must_use]
    pub fn new(status: StatusCode, code: FfiErrorCode, message: impl Into<String>) -> Self {
        ApiError {
            status,
            code: code.as_i32(),
            message: message.into(),
        }
    }

    /// The failure of a name that is not a protocol.
    #[must_use]
    pub fn unknown_protocol(name: &str) -> Self {
        ApiError::new(
            StatusCode::NOT_FOUND,
            FfiErrorCode::InvalidInput,
            format!("unknown protocol `{name}`"),
        )
    }

    /// The failure of a request that could not be understood.
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        ApiError::new(StatusCode::BAD_REQUEST, FfiErrorCode::InvalidInput, message)
    }

    /// The failure of a configuration that the receiver refuses.
    #[must_use]
    pub fn invalid_configuration(message: impl Into<String>) -> Self {
        ApiError::new(StatusCode::BAD_REQUEST, FfiErrorCode::InvalidInput, message)
    }

    /// The failure of the receiver itself.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            FfiErrorCode::Internal,
            message,
        )
    }
}

impl From<FfiError> for ApiError {
    fn from(error: FfiError) -> Self {
        // WHY: the code of the bridge is the code of the core. A failure
        // that the bridge already classified keeps its classification.
        let status = match FfiErrorCode::from_i32(error.code()) {
            Some(FfiErrorCode::InvalidInput) => StatusCode::BAD_REQUEST,
            Some(FfiErrorCode::Unsupported) => StatusCode::NOT_IMPLEMENTED,
            Some(FfiErrorCode::InvalidState) => StatusCode::CONFLICT,
            Some(FfiErrorCode::Timeout) => StatusCode::GATEWAY_TIMEOUT,
            Some(FfiErrorCode::ResourceExhausted) => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        ApiError {
            status,
            code: error.code(),
            message: error.details().to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            error: ErrorBody {
                code: self.code,
                message: self.message,
            },
        };
        (self.status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phairplay_ffi::error::FfiError;

    #[test]
    fn an_unknown_protocol_is_a_not_found() {
        let error = ApiError::unknown_protocol("telepathy");
        assert_eq!(error.status, StatusCode::NOT_FOUND);
        assert_eq!(error.code, FfiErrorCode::InvalidInput.as_i32());
        assert!(error.message.contains("telepathy"));
    }

    #[test]
    fn an_invalid_request_is_a_bad_request() {
        let error = ApiError::bad_request("the body is not a configuration");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn the_code_of_the_bridge_is_kept() {
        let error = ApiError::from(FfiError::invalid_input("the name is wrong"));
        assert_eq!(error.code, FfiErrorCode::InvalidInput.as_i32());
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "the name is wrong");
    }

    #[test]
    fn an_unsupported_feature_is_a_not_implemented() {
        let error = ApiError::from(FfiError::unsupported("there is no engine"));
        assert_eq!(error.status, StatusCode::NOT_IMPLEMENTED);
    }
}
