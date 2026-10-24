
use super::*;

use crate::error::AirPlayError;

#[test]
fn a_command_has_a_stable_path() {
    assert_eq!(DacpCommand::PlayPause.path(), "/ctrl-int/1/playpause");
    assert_eq!(DacpCommand::NextItem.path(), "/ctrl-int/1/nextitem");
    assert_eq!(DacpCommand::MuteToggle.path(), "/ctrl-int/1/mutetoggle");
}

#[test]
fn the_identity_is_stored_once() {
    let mut client = DacpClient::new();
    assert!(client.configure(Some("0A1B"), Some("token")));
    assert!(!client.configure(Some("0A1B"), Some("token")));
    assert!(client.is_available());
}

#[test]
fn an_empty_identity_is_ignored() {
    let mut client = DacpClient::new();
    assert!(!client.configure(Some("  "), Some("token")));
    assert!(!client.configure(Some("0A1B"), None));
    assert!(!client.is_available());
}

#[test]
fn the_service_name_carries_the_identifier() {
    let mut client = DacpClient::new();
    client.configure(Some("0A1B2C3D"), Some("token"));
    let identity = client.identity().expect("identity");
    assert_eq!(identity.service_name(), "iTunes_Ctrl_0A1B2C3D");
}

#[test]
fn the_request_carries_the_token() {
    let mut client = DacpClient::new();
    client.configure(Some("0A1B"), Some("secret-token"));
    let request = client
        .build_request(DacpCommand::PlayPause)
        .expect("request");
    let text = String::from_utf8(request).expect("utf-8");
    assert!(text.starts_with("GET /ctrl-int/1/playpause HTTP/1.1\r\n"), "{text}");
    assert!(text.contains("Active-Remote: secret-token\r\n"), "{text}");
}

#[test]
fn a_request_without_an_identity_is_rejected() {
    let client = DacpClient::new();
    let error = client
        .build_request(DacpCommand::PlayPause)
        .expect_err("must fail");
    assert!(matches!(error, AirPlayError::InvalidState { .. }));
}

#[tokio::test]
async fn a_command_without_an_endpoint_is_rejected() {
    let mut client = DacpClient::new();
    client.configure(Some("0A1B"), Some("token"));
    let error = client
        .send_command(DacpCommand::PlayPause)
        .await
        .expect_err("must fail");
    assert!(matches!(error, AirPlayError::InvalidState { .. }));
}

#[test]
fn the_status_code_is_parsed() {
    assert_eq!(parse_status("HTTP/1.1 200 OK\r\n\r\n"), 200);
    assert_eq!(parse_status("garbage"), 0);
}
