#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

#[test]
fn a_renderer_search_is_recognised() {
    let message = "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\n\
                   MAN: \"ssdp:discover\"\r\nMX: 1\r\nST: urn:schemas-upnp-org:device:MediaRenderer:1\r\n\r\n";
    assert!(is_renderer_search(message));
}

#[test]
fn a_search_for_the_root_device_is_recognised() {
    let message = "M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\nST: upnp:rootdevice\r\n\r\n";
    assert!(is_renderer_search(message));
}

#[test]
fn a_search_for_everything_is_recognised() {
    let message = "M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\nST: ssdp:all\r\n\r\n";
    assert!(is_renderer_search(message));
}

#[test]
fn a_search_for_another_device_is_ignored() {
    let message = "M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\nST: urn:schemas-upnp-org:device:MediaServer:1\r\n\r\n";
    assert!(!is_renderer_search(message));
}

#[test]
fn a_message_that_is_not_a_search_is_ignored() {
    assert!(!is_renderer_search("NOTIFY * HTTP/1.1\r\n\r\n"));
    assert!(!is_renderer_search("GET / HTTP/1.1\r\n\r\n"));
    assert!(!is_renderer_search(""));
}

#[test]
fn a_search_without_the_man_header_is_ignored() {
    let message = "M-SEARCH * HTTP/1.1\r\nST: ssdp:all\r\n\r\n";
    assert!(!is_renderer_search(message));
}

#[test]
fn the_search_target_is_read() {
    let message = "M-SEARCH * HTTP/1.1\r\nST: urn:schemas-upnp-org:device:MediaRenderer:1\r\n\r\n";
    assert_eq!(requested_target(message), MEDIA_RENDERER);
}

#[test]
fn a_search_for_everything_is_answered_as_a_renderer() {
    let message = "M-SEARCH * HTTP/1.1\r\nST: ssdp:all\r\n\r\n";
    assert_eq!(requested_target(message), MEDIA_RENDERER);
}

#[test]
fn a_search_without_a_target_falls_back_to_the_renderer() {
    let message = "M-SEARCH * HTTP/1.1\r\nMX: 1\r\n\r\n";
    assert_eq!(requested_target(message), MEDIA_RENDERER);
}

#[test]
fn the_search_response_carries_the_location_and_the_usn() {
    let response = search_response(
        "http://10.0.0.5:8200/description.xml",
        "uuid:abc",
        "ssdp:all",
    );
    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response.contains("LOCATION: http://10.0.0.5:8200/description.xml\r\n"));
    assert!(response.contains("USN: uuid:abc::ssdp:all\r\n"));
    assert!(response.contains("CACHE-CONTROL: max-age=1800"));
    assert!(response.ends_with("\r\n\r\n"));
}

#[test]
fn the_alive_announcement_is_complete() {
    let message = notify_alive(
        "http://10.0.0.5:8200/description.xml",
        "uuid:abc",
        "upnp:rootdevice",
    );
    assert!(message.starts_with("NOTIFY * HTTP/1.1\r\n"));
    assert!(message.contains("HOST: 239.255.255.250:1900\r\n"));
    assert!(message.contains("NT: upnp:rootdevice\r\n"));
    assert!(message.contains("NTS: ssdp:alive\r\n"));
    assert!(message.contains("USN: uuid:abc::upnp:rootdevice\r\n"));
}

#[test]
fn the_byebye_announcement_is_complete() {
    let message = notify_byebye("uuid:abc", "upnp:rootdevice");
    assert!(message.starts_with("NOTIFY * HTTP/1.1\r\n"));
    assert!(message.contains("NTS: ssdp:byebye\r\n"));
    assert!(!message.contains("LOCATION"));
}

#[test]
fn the_renderer_announces_itself_under_three_targets() {
    let targets = notification_targets("uuid:abc");
    assert_eq!(targets.len(), 3);
    assert!(targets.contains(&ROOT_DEVICE.to_string()));
    assert!(targets.contains(&MEDIA_RENDERER.to_string()));
    assert!(targets.contains(&"uuid:abc".to_string()));
}

#[test]
fn the_responder_answers_a_query_on_its_own_port() {
    // The responder is bound to an ephemeral port so that the test does
    // not fight with a real DLNA server on the machine.
    let mut responder = SsdpResponder::bind(
        0,
        "http://127.0.0.1:1/d.xml".to_string(),
        "uuid:abc".to_string(),
    )
    .expect("the responder binds");
    let port = responder.local_port();
    assert!(port > 0);
    let client =
        UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).expect("the client binds");
    client
        .send_to(
            b"M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\nST: ssdp:all\r\n\r\n",
            SocketAddr::from((Ipv4Addr::LOCALHOST, port)),
        )
        .expect("the query is sent");
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("the timeout is set");
    let mut buffer = [0u8; 1024];
    let (read, _) = client.recv_from(&mut buffer).expect("an answer arrives");
    let answer = String::from_utf8_lossy(&buffer[..read]).to_string();
    assert!(answer.starts_with("HTTP/1.1 200 OK"), "{answer}");
    assert!(answer.contains("LOCATION: http://127.0.0.1:1/d.xml"));
    responder.stop();
}
