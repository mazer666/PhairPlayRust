#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

#[test]
fn the_method_names_match_the_wire() {
    assert_eq!(Method::Get.as_str(), "GET");
    assert_eq!(Method::Subscribe.as_str(), "SUBSCRIBE");
    assert_eq!(Method::Other("PATCH".to_string()).as_str(), "PATCH");
}

#[test]
fn the_error_statuses_match_the_protocol() {
    assert_eq!(HttpError::LineTooLong { limit: 8 }.status(), 414);
    assert_eq!(HttpError::TooManyHeaders { limit: 8 }.status(), 431);
    assert_eq!(HttpError::BodyTooLarge { limit: 8 }.status(), 413);
    assert_eq!(HttpError::Malformed("x".to_string()).status(), 400);
    assert_eq!(HttpError::Timeout.status(), 408);
    assert_eq!(HttpError::UnsupportedTransferEncoding.status(), 501);
    assert_eq!(HttpError::LengthRequired.status(), 411);
}

#[test]
fn the_default_limits_are_bounded() {
    let limits = HttpLimits::default();
    assert!(limits.max_header_count > 0);
    assert!(limits.max_header_bytes > 0);
    assert!(limits.max_body_bytes > 0);
    assert!(limits.read_timeout > Duration::ZERO);
}
