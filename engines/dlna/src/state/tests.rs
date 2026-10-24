#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

fn peer(port: u16) -> SocketAddr {
    SocketAddr::from(([192, 168, 1, 20], port))
}

#[test]
fn a_new_renderer_has_no_media() {
    let state = RendererState::new();
    assert_eq!(state.transport_state(), TransportState::NoMediaPresent);
    assert!(state.current_uri().is_empty());
    assert_eq!(state.volume(), DEFAULT_VOLUME);
    assert!(!state.muted());
}

#[test]
fn the_first_peer_claims_the_session() {
    let state = RendererState::new();
    let id = state.claim_session(peer(1)).expect("the slot is free");
    assert!(id > 0);
    assert_eq!(state.session().map(|s| s.id), Some(id));
}

#[test]
fn the_same_peer_may_claim_the_session_again() {
    let state = RendererState::new();
    let first = state.claim_session(peer(1)).expect("claimed");
    let second = state
        .claim_session(peer(1))
        .expect("the same peer continues");
    assert_eq!(first, second);
}

#[test]
fn a_second_peer_is_rejected_while_a_stream_runs() {
    let state = RendererState::new();
    state.claim_session(peer(1)).expect("claimed");
    state.set_media("http://10.0.0.1/movie.mp4", "");
    state.set_transport_state(TransportState::Playing);
    let error = state
        .claim_session(peer(2))
        .expect_err("the second peer is rejected");
    assert_eq!(error, SessionError::AlreadyActive);
}

#[test]
fn a_second_peer_may_take_over_when_nothing_is_playing() {
    let state = RendererState::new();
    state.claim_session(peer(1)).expect("claimed");
    state.set_media("http://10.0.0.1/movie.mp4", "");
    state.set_transport_state(TransportState::Stopped);
    let id = state
        .claim_session(peer(2))
        .expect("the slot is free again");
    assert!(id > 0);
}

#[test]
fn stopping_releases_the_session() {
    let state = RendererState::new();
    state.claim_session(peer(1)).expect("claimed");
    assert!(state.release_session(peer(1)));
    assert!(state.session().is_none());
}

#[test]
fn a_peer_may_not_release_the_session_of_another_peer() {
    let state = RendererState::new();
    state.claim_session(peer(1)).expect("claimed");
    assert!(!state.release_session(peer(2)));
    assert!(state.session().is_some());
}

#[test]
fn setting_the_media_resets_the_position() {
    let state = RendererState::new();
    state.set_position(Duration::from_secs(30));
    state.set_media("http://10.0.0.1/movie.mp4", "<meta/>");
    assert_eq!(state.position(), Duration::ZERO);
    assert_eq!(state.current_uri(), "http://10.0.0.1/movie.mp4");
    assert_eq!(state.current_uri_metadata(), "<meta/>");
    assert_eq!(state.transport_state(), TransportState::Stopped);
}

#[test]
fn the_volume_is_clamped() {
    let state = RendererState::new();
    state.set_volume(200);
    assert_eq!(state.volume(), MAX_VOLUME);
    state.set_volume(0);
    assert_eq!(state.volume(), 0);
}

#[test]
fn the_position_is_clamped_to_a_known_duration() {
    let state = RendererState::new();
    state.set_position(Duration::from_secs(30));
    // Without a known duration the position is kept.
    assert_eq!(state.position(), Duration::from_secs(30));
}

#[test]
fn the_transport_state_names_match_the_specification() {
    assert_eq!(TransportState::Stopped.as_str(), "STOPPED");
    assert_eq!(TransportState::Playing.as_str(), "PLAYING");
    assert_eq!(TransportState::PausedPlayback.as_str(), "PAUSED_PLAYBACK");
    assert_eq!(TransportState::Transitioning.as_str(), "TRANSITIONING");
    assert_eq!(TransportState::NoMediaPresent.as_str(), "NO_MEDIA_PRESENT");
}

#[test]
fn only_an_active_transport_blocks_a_second_peer() {
    assert!(!TransportState::Stopped.is_active());
    assert!(!TransportState::NoMediaPresent.is_active());
    assert!(TransportState::Playing.is_active());
    assert!(TransportState::PausedPlayback.is_active());
}
