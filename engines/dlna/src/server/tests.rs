#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;
use crate::description::{
    CONTROL_PATH_AV_TRANSPORT, CONTROL_PATH_CONNECTION_MANAGER, CONTROL_PATH_RENDERING_CONTROL,
    EVENT_PATH_AV_TRANSPORT, EVENT_PATH_CONNECTION_MANAGER, EVENT_PATH_RENDERING_CONTROL,
};
use routing::{is_control_path, is_event_path};

#[test]
fn the_lifecycle_starts_idle() {
    let lifecycle = Lifecycle::new();
    assert_eq!(lifecycle.state(), ProtocolState::Idle);
    assert_eq!(lifecycle.connections(), 0);
    assert!(lifecycle.tcp_port().is_none());
    assert!(lifecycle.enabled());
}

#[test]
fn the_connection_counter_never_goes_below_zero() {
    let mut lifecycle = Lifecycle::new();
    lifecycle.connection_closed();
    assert_eq!(lifecycle.connections(), 0);
    lifecycle.connection_opened();
    lifecycle.connection_opened();
    assert_eq!(lifecycle.connections(), 2);
    lifecycle.connection_closed();
    assert_eq!(lifecycle.connections(), 1);
}

#[test]
fn the_event_paths_are_recognised() {
    assert!(is_event_path(EVENT_PATH_AV_TRANSPORT));
    assert!(is_event_path(EVENT_PATH_CONNECTION_MANAGER));
    assert!(is_event_path(EVENT_PATH_RENDERING_CONTROL));
    assert!(!is_event_path("/upnp/control/AVTransport"));
}

#[test]
fn the_control_paths_are_recognised() {
    assert!(is_control_path(CONTROL_PATH_AV_TRANSPORT));
    assert!(is_control_path(CONTROL_PATH_CONNECTION_MANAGER));
    assert!(is_control_path(CONTROL_PATH_RENDERING_CONTROL));
    assert!(!is_control_path("/upnp/event/AVTransport"));
}
