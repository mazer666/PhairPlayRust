//! The tests of the single-session registry.
//!
//! WHY: the rule "one peer at a time" is the rule that keeps a Smart View
//! receiver from being taken over by a second device. The tests therefore
//! cover the rejection and not only the happy path.

#![allow(clippy::expect_used, clippy::unwrap_used)]
use super::*;
use phairplay_core::protocol::PeerLimit;

/// The address of a peer on the loopback interface.
fn peer_address() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 40_000))
}

#[test]
fn an_empty_registry_has_no_session() {
    let registry = SessionRegistry::new();
    assert!(registry.active().is_none());
    assert!(!registry.is_active());
}

#[test]
fn a_session_is_opened_for_the_first_peer() {
    let registry = SessionRegistry::new();
    let session = registry
        .open(peer_address(), PeerLimit::One)
        .expect("the session opens");
    assert_eq!(session.address, peer_address());
    assert!(registry.is_active());
    assert_eq!(registry.active().map(|active| active.id), Some(session.id));
}

#[test]
fn a_second_peer_is_rejected_and_not_queued() {
    // WHY: a queued peer would wait for a session that may never end.
    let registry = SessionRegistry::new();
    let _first = registry
        .open(peer_address(), PeerLimit::One)
        .expect("the session opens");
    let error = registry
        .open(SocketAddr::from(([127, 0, 0, 2], 40_001)), PeerLimit::One)
        .expect_err("the second peer is refused");
    assert!(error.to_string().contains("already open"), "{error}");
    assert!(error.is_peer_fault());
    // The first session is untouched.
    assert_eq!(
        registry.active().map(|active| active.address),
        Some(peer_address())
    );
}

#[test]
fn a_closed_session_frees_the_slot() {
    let registry = SessionRegistry::new();
    let session = registry
        .open(peer_address(), PeerLimit::One)
        .expect("the session opens");
    let closed = registry.close(session.id).expect("the session closes");
    assert_eq!(closed.id, session.id);
    assert!(!registry.is_active());
    let _second = registry
        .open(SocketAddr::from(([127, 0, 0, 2], 40_001)), PeerLimit::One)
        .expect("the next peer is accepted");
}

#[test]
fn a_late_close_of_an_old_session_does_not_release_the_new_one() {
    let registry = SessionRegistry::new();
    let first = registry
        .open(peer_address(), PeerLimit::One)
        .expect("the session opens");
    registry.close(first.id).expect("the session closes");
    let second = registry
        .open(SocketAddr::from(([127, 0, 0, 2], 40_001)), PeerLimit::One)
        .expect("the next peer is accepted");
    assert!(registry.close(first.id).is_none());
    assert!(registry.is_active());
    assert_eq!(registry.active().map(|active| active.id), Some(second.id));
}

#[test]
fn the_announced_peer_is_used_by_the_session() {
    // WHY: the sender announces itself before it opens the channel.
    let registry = SessionRegistry::new();
    registry.announce(PeerInfo {
        mac: Some("aabbccddeeff".to_string()),
        ip: Some("192.168.1.20".to_string()),
        name: Some("Galaxy S23".to_string()),
    });
    let session = registry
        .open(peer_address(), PeerLimit::One)
        .expect("the session opens");
    assert_eq!(session.peer.name.as_deref(), Some("Galaxy S23"));
    assert_eq!(session.peer.mac.as_deref(), Some("aabbccddeeff"));
}

#[test]
fn a_peer_without_an_announcement_still_opens_a_session() {
    let registry = SessionRegistry::new();
    let session = registry
        .open(peer_address(), PeerLimit::One)
        .expect("the session opens");
    assert!(session.peer.name.is_none());
    assert_eq!(session.peer.display_name(), "Unknown sender");
}

#[test]
fn the_log_summary_carries_the_address_and_the_mac() {
    let peer = PeerInfo {
        mac: Some("aabbccddeeff".to_string()),
        ip: Some("192.168.1.20".to_string()),
        name: None,
    };
    let summary = peer.log_summary();
    assert!(summary.contains("192.168.1.20"), "{summary}");
    assert!(summary.contains("aabbccddeeff"), "{summary}");
    assert!(!summary.contains("Galaxy"), "{summary}");
}

#[test]
fn an_empty_name_falls_back() {
    let peer = PeerInfo {
        name: Some("   ".to_string()),
        ..PeerInfo::default()
    };
    assert_eq!(peer.display_name(), "Unknown sender");
}
