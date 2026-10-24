use super::*;

#[test]
fn a_new_session_carries_no_media() {
    let session = AirPlaySession::new();
    assert!(!session.has_video());
    assert!(!session.has_audio());
    assert!(!session.is_streaming());
}

#[test]
fn the_stream_types_are_tracked() {
    let mut session = AirPlaySession::new();
    session.add_stream_type(STREAM_TYPE_VIDEO);
    session.add_stream_type(STREAM_TYPE_AUDIO);
    assert!(!session.has_no_active_streams());
    session.remove_stream_type(STREAM_TYPE_AUDIO);
    assert!(!session.has_no_active_streams());
    session.remove_stream_type(STREAM_TYPE_VIDEO);
    assert!(session.has_no_active_streams());
}

#[test]
fn the_sender_name_falls_back_to_a_generic_one() {
    let session = AirPlaySession::new();
    assert_eq!(session.sender_name(), "AirPlay Sender");
}

#[test]
fn every_session_has_its_own_identifier() {
    let first = AirPlaySession::new();
    let second = AirPlaySession::new();
    assert_ne!(first.id, second.id);
}
