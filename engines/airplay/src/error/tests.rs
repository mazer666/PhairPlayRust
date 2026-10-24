
use super::*;

use crate::error::AirPlayError;
use phairplay_core::error::ErrorKind;

#[test]
fn a_limit_error_maps_onto_the_core_limit_variant() {
    let error: EngineError = AirPlayError::limit_exceeded("header count").into();
    assert!(matches!(error, EngineError::LimitExceeded { .. }));
}

#[test]
fn a_protocol_violation_maps_onto_the_core_protocol_variant() {
    let error: EngineError = AirPlayError::protocol("bad cseq").into();
    assert!(matches!(error, EngineError::Protocol { .. }));
}

#[test]
fn the_core_error_kind_of_a_limit_is_resource_exhausted() {
    let error: phairplay_core::error::CoreError =
        AirPlayError::limit_exceeded("body").into();
    assert_eq!(ErrorKind::classify(&error), ErrorKind::ResourceExhausted);
}

#[test]
fn an_unsupported_error_still_names_the_boundary() {
    let text = AirPlayError::unsupported("FairPlay stream key").to_string();
    assert!(text.contains("FairPlay stream key"), "{text}");
}

#[test]
fn an_io_error_is_carried_through() {
    let io = std::io::Error::new(std::io::ErrorKind::ConnectionReset, "reset");
    let error = AirPlayError::from(io);
    assert!(error.to_string().contains("reset"), "{error}");
}
