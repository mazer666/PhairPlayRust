//! The tests of the lifecycle of the engine.
//!
//! WHY: the lifecycle is what the dashboard, the metrics and the bridge
//! read. Every accessor is a promise about the state of the receiver, and
//! every promise needs a test.

use super::*;
use phairplay_core::protocol::ProtocolState;

#[test]
fn a_new_lifecycle_is_idle_and_serves_no_port() {
    let lifecycle = Lifecycle::new();
    assert_eq!(lifecycle.state(), ProtocolState::Idle);
    assert_eq!(lifecycle.tcp_port(), None);
    assert_eq!(lifecycle.udp_port(), None);
    assert_eq!(lifecycle.last_error(), None);
    assert!(lifecycle.enabled());
}

#[test]
fn the_state_is_written_and_read_back() {
    let mut lifecycle = Lifecycle::new();
    lifecycle.set_state(ProtocolState::Advertising);
    assert_eq!(lifecycle.state(), ProtocolState::Advertising);
    lifecycle.set_state(ProtocolState::Stopped);
    assert_eq!(lifecycle.state(), ProtocolState::Stopped);
}

#[test]
fn the_tcp_port_is_written_and_read_back() {
    let mut lifecycle = Lifecycle::new();
    lifecycle.set_tcp_port(Some(8001));
    assert_eq!(lifecycle.tcp_port(), Some(8001));
    lifecycle.set_tcp_port(None);
    assert_eq!(lifecycle.tcp_port(), None);
}

#[test]
fn the_udp_port_is_written_and_read_back() {
    let mut lifecycle = Lifecycle::new();
    lifecycle.set_udp_port(Some(1900));
    assert_eq!(lifecycle.udp_port(), Some(1900));
    lifecycle.set_udp_port(None);
    assert_eq!(lifecycle.udp_port(), None);
}

#[test]
fn the_reason_of_the_last_failure_is_kept() {
    let mut lifecycle = Lifecycle::new();
    lifecycle.set_last_error(Some("the port 1900 is already in use".to_string()));
    assert_eq!(
        lifecycle.last_error(),
        Some("the port 1900 is already in use".to_string())
    );
    lifecycle.set_last_error(None);
    assert_eq!(lifecycle.last_error(), None);
}

#[test]
fn the_protocol_can_be_disabled() {
    let mut lifecycle = Lifecycle::new();
    lifecycle.set_enabled(false);
    assert!(!lifecycle.enabled());
    lifecycle.set_enabled(true);
    assert!(lifecycle.enabled());
}
