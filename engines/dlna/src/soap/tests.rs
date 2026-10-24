#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;
use crate::description::{
    SERVICE_AV_TRANSPORT, SERVICE_CONNECTION_MANAGER, SERVICE_RENDERING_CONTROL, advertised_actions,
};
use std::time::Duration;

fn peer() -> SocketAddr {
    SocketAddr::from(([192, 168, 1, 20], 50_000))
}

fn dispatcher() -> SoapDispatcher {
    SoapDispatcher::new(Arc::new(RendererState::new()), Arc::new(NoEvents))
}

fn request(control_path: &str, action: &str, arguments: &str) -> SoapRequest {
    SoapRequest {
        control_path: control_path.to_string(),
        soap_action: Some(format!("urn:schemas-upnp-org:service:X:1#{action}")),
        body: format!(
            "<s:Envelope xmlns:s=\"urn:x\"><s:Body><u:{action} xmlns:u=\"urn:y\">{arguments}</u:{action}></s:Body></s:Envelope>"
        ),
        peer: peer(),
    }
}

#[test]
fn the_protocol_info_is_announced() {
    let response = dispatcher().dispatch(&request(
        CONTROL_PATH_CONNECTION_MANAGER,
        "GetProtocolInfo",
        "",
    ));
    assert_eq!(response.status, 200);
    assert!(response.body.contains(SINK_PROTOCOL_INFO));
}

#[test]
fn an_unknown_action_is_reported_as_a_fault() {
    let response = dispatcher().dispatch(&request(CONTROL_PATH_CONNECTION_MANAGER, "Teleport", ""));
    assert_eq!(response.status, 500);
    assert!(response.body.contains("<errorCode>401</errorCode>"));
}

#[test]
fn a_doctype_in_the_body_is_rejected() {
    let mut request = request(CONTROL_PATH_AV_TRANSPORT, "Play", "");
    request.body =
        "<?xml version=\"1.0\"?><!DOCTYPE x [<!ENTITY e SYSTEM \"file:///etc/passwd\">]>\
                    <s:Envelope xmlns:s=\"urn:x\"><s:Body><u:Play/></s:Body></s:Envelope>"
            .to_string();
    let response = dispatcher().dispatch(&request);
    assert_eq!(response.status, 400);
    assert!(!response.body.contains("root:"));
}

#[test]
fn an_entity_in_the_body_is_rejected() {
    let mut request = request(CONTROL_PATH_AV_TRANSPORT, "Play", "");
    request.body =
        "<s:Envelope xmlns:s=\"urn:x\"><s:Body><u:Play>&xxe;</u:Play></s:Body></s:Envelope>"
            .to_string();
    let response = dispatcher().dispatch(&request);
    assert_eq!(response.status, 400);
}

#[test]
fn a_mismatching_action_header_is_rejected() {
    let mut request = request(CONTROL_PATH_AV_TRANSPORT, "Play", "");
    request.soap_action = Some("urn:x#Stop".to_string());
    let response = dispatcher().dispatch(&request);
    assert_eq!(response.status, 400);
}

#[test]
fn the_action_is_taken_from_the_body_when_the_header_is_missing() {
    let mut request = request(CONTROL_PATH_AV_TRANSPORT, "GetTransportInfo", "");
    request.soap_action = None;
    let response = dispatcher().dispatch(&request);
    assert_eq!(response.status, 200);
    assert!(response.body.contains("CurrentTransportState"));
}

#[test]
fn the_media_is_set_and_reported() {
    let dispatcher = dispatcher();
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "SetAVTransportURI",
        "<InstanceID>0</InstanceID><CurrentURI>http://10.0.0.9/movie.mp4</CurrentURI>",
    ));
    assert_eq!(response.status, 200);
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "GetMediaInfo",
        "<InstanceID>0</InstanceID>",
    ));
    assert!(response.body.contains("http://10.0.0.9/movie.mp4"));
    assert!(response.body.contains("<NrTracks>1</NrTracks>"));
}

#[test]
fn an_empty_current_uri_is_rejected() {
    let response = dispatcher().dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "SetAVTransportURI",
        "<InstanceID>0</InstanceID><CurrentURI></CurrentURI>",
    ));
    assert_eq!(response.status, 500);
    assert!(response.body.contains("<errorCode>402</errorCode>"));
}

#[test]
fn playing_without_media_is_refused() {
    let response = dispatcher().dispatch(&request(CONTROL_PATH_AV_TRANSPORT, "Play", ""));
    assert_eq!(response.status, 500);
    assert!(response.body.contains("<errorCode>701</errorCode>"));
}

#[test]
fn the_transport_state_follows_the_actions() {
    let dispatcher = dispatcher();
    let _ = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "SetAVTransportURI",
        "<CurrentURI>http://10.0.0.9/movie.mp4</CurrentURI>",
    ));
    let _ = dispatcher.dispatch(&request(CONTROL_PATH_AV_TRANSPORT, "Play", ""));
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "GetTransportInfo",
        "<InstanceID>0</InstanceID>",
    ));
    assert!(
        response
            .body
            .contains("<CurrentTransportState>PLAYING</CurrentTransportState>")
    );

    let _ = dispatcher.dispatch(&request(CONTROL_PATH_AV_TRANSPORT, "Pause", ""));
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "GetTransportInfo",
        "<InstanceID>0</InstanceID>",
    ));
    assert!(
        response
            .body
            .contains("<CurrentTransportState>PAUSED_PLAYBACK</CurrentTransportState>")
    );

    let _ = dispatcher.dispatch(&request(CONTROL_PATH_AV_TRANSPORT, "Stop", ""));
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "GetTransportInfo",
        "<InstanceID>0</InstanceID>",
    ));
    assert!(
        response
            .body
            .contains("<CurrentTransportState>STOPPED</CurrentTransportState>")
    );
}

#[test]
fn a_second_control_point_is_rejected_while_the_first_streams() {
    let dispatcher = dispatcher();
    let _ = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "SetAVTransportURI",
        "<CurrentURI>http://10.0.0.9/movie.mp4</CurrentURI>",
    ));
    let _ = dispatcher.dispatch(&request(CONTROL_PATH_AV_TRANSPORT, "Play", ""));

    let mut second = request(
        CONTROL_PATH_AV_TRANSPORT,
        "SetAVTransportURI",
        "<CurrentURI>http://10.0.0.8/other.mp4</CurrentURI>",
    );
    second.peer = SocketAddr::from(([192, 168, 1, 21], 50_000));
    let response = dispatcher.dispatch(&second);
    assert_eq!(response.status, 503);
    assert!(response.body.contains("<errorCode>701</errorCode>"));
}

#[test]
fn the_same_control_point_may_continue() {
    let dispatcher = dispatcher();
    let _ = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "SetAVTransportURI",
        "<CurrentURI>http://10.0.0.9/movie.mp4</CurrentURI>",
    ));
    let response = dispatcher.dispatch(&request(CONTROL_PATH_AV_TRANSPORT, "Play", ""));
    assert_eq!(response.status, 200);
}

#[test]
fn the_volume_is_set_and_clamped() {
    let dispatcher = dispatcher();
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_RENDERING_CONTROL,
        "SetVolume",
        "<InstanceID>0</InstanceID><Channel>Master</Channel><DesiredVolume>70</DesiredVolume>",
    ));
    assert_eq!(response.status, 200);
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_RENDERING_CONTROL,
        "GetVolume",
        "<InstanceID>0</InstanceID><Channel>Master</Channel>",
    ));
    assert!(response.body.contains("<CurrentVolume>70</CurrentVolume>"));
}

#[test]
fn a_volume_that_is_not_a_number_is_refused() {
    let response = dispatcher().dispatch(&request(
        CONTROL_PATH_RENDERING_CONTROL,
        "SetVolume",
        "<DesiredVolume>loud</DesiredVolume>",
    ));
    assert_eq!(response.status, 500);
    assert!(response.body.contains("<errorCode>402</errorCode>"));
}

#[test]
fn a_volume_out_of_range_is_refused() {
    let response = dispatcher().dispatch(&request(
        CONTROL_PATH_RENDERING_CONTROL,
        "SetVolume",
        "<DesiredVolume>500</DesiredVolume>",
    ));
    assert_eq!(response.status, 500);
}

#[test]
fn the_mute_state_is_set() {
    let dispatcher = dispatcher();
    let _ = dispatcher.dispatch(&request(
        CONTROL_PATH_RENDERING_CONTROL,
        "SetMute",
        "<DesiredMute>1</DesiredMute>",
    ));
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_RENDERING_CONTROL,
        "GetMute",
        "<InstanceID>0</InstanceID><Channel>Master</Channel>",
    ));
    assert!(response.body.contains("<CurrentMute>1</CurrentMute>"));
}

#[test]
fn a_mute_value_that_is_not_a_boolean_is_refused() {
    let response = dispatcher().dispatch(&request(
        CONTROL_PATH_RENDERING_CONTROL,
        "SetMute",
        "<DesiredMute>maybe</DesiredMute>",
    ));
    assert_eq!(response.status, 500);
}

#[test]
fn an_unknown_control_path_is_a_not_found() {
    let response = dispatcher().dispatch(&request("/upnp/control/Telepathy", "Play", ""));
    assert_eq!(response.status, 404);
}

#[test]
fn a_body_without_an_action_is_rejected() {
    let mut request = request(CONTROL_PATH_AV_TRANSPORT, "Play", "");
    request.body = "<s:Envelope xmlns:s=\"urn:x\"><s:Body/></s:Envelope>".to_string();
    request.soap_action = None;
    let response = dispatcher().dispatch(&request);
    assert_eq!(response.status, 400);
}

#[test]
fn a_seek_sets_the_position() {
    let dispatcher = dispatcher();
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "Seek",
        "<Unit>REL_TIME</Unit><Target>0:01:30</Target>",
    ));
    assert_eq!(response.status, 200);
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "GetPositionInfo",
        "<InstanceID>0</InstanceID>",
    ));
    assert!(response.body.contains("<RelTime>00:01:30</RelTime>"));
}

#[test]
fn a_seek_with_an_unknown_unit_is_refused() {
    let response = dispatcher().dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "Seek",
        "<Unit>XYZZY</Unit><Target>0:01:30</Target>",
    ));
    assert_eq!(response.status, 500);
    assert!(response.body.contains("<errorCode>710</errorCode>"));
}

#[test]
fn a_seek_with_a_malformed_target_is_refused() {
    let response = dispatcher().dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "Seek",
        "<Unit>REL_TIME</Unit><Target>later</Target>",
    ));
    assert_eq!(response.status, 500);
    assert!(response.body.contains("<errorCode>402</errorCode>"));
}

#[test]
fn the_transport_actions_follow_the_state() {
    let dispatcher = dispatcher();
    let response = dispatcher.dispatch(&request(
        CONTROL_PATH_AV_TRANSPORT,
        "GetCurrentTransportActions",
        "<InstanceID>0</InstanceID>",
    ));
    assert!(response.body.contains("<Actions>Play,Stop</Actions>"));
}

#[test]
fn every_advertised_action_is_implemented() {
    // A control point that calls an advertised action and gets a fault
    // has found a broken renderer. This test walks the advertisement.
    let dispatcher = dispatcher();
    let services = [
        (CONTROL_PATH_CONNECTION_MANAGER, SERVICE_CONNECTION_MANAGER),
        (CONTROL_PATH_AV_TRANSPORT, SERVICE_AV_TRANSPORT),
        (CONTROL_PATH_RENDERING_CONTROL, SERVICE_RENDERING_CONTROL),
    ];
    for (control_path, service) in services {
        for action in advertised_actions(service) {
            let response = dispatcher.dispatch(&request(control_path, action, ""));
            assert!(
                !response.body.contains("<errorCode>401</errorCode>"),
                "{service}/{action} is advertised but not implemented"
            );
        }
    }
}

#[test]
fn the_soap_action_header_is_parsed() {
    assert_eq!(
        parse_soap_action("\"urn:schemas-upnp-org:service:AVTransport:1#Play\""),
        Some("Play".to_string())
    );
    assert_eq!(parse_soap_action("Play"), Some("Play".to_string()));
    assert_eq!(parse_soap_action(""), None);
    assert_eq!(parse_soap_action("\"Play; \""), None);
}

#[test]
fn a_time_is_parsed_and_formatted() {
    assert_eq!(parse_time("0:01:30"), Some(Duration::from_secs(90)));
    assert_eq!(parse_time("1:00:00"), Some(Duration::from_secs(3600)));
    assert_eq!(parse_time("0:00:01.5"), Some(Duration::from_millis(1500)));
    assert_eq!(parse_time("later"), None);
    assert_eq!(parse_time("0:99:00"), None);
    assert_eq!(format_duration(Duration::from_secs(90)), "00:01:30");
}
