//! The tests of the SSDP announcement.
//!
//! WHY: the codec decides which device shows up in the list of a Galaxy
//! phone. A search that is answered although it asks for another service
//! puts the receiver into every device list of the network; a search that is
//! not answered although it asks for the MultiScreen service makes the
//! receiver invisible.

#![allow(clippy::expect_used, clippy::unwrap_used)]
use super::*;

/// A well-formed search of a Galaxy device.
fn search() -> String {
    format!(
        "M-SEARCH * HTTP/1.1\r\nHOST: {MULTICAST_ADDRESS}:{SSDP_PORT}\r\n\
         MAN: \"ssdp:discover\"\r\nMX: 3\r\nST: {SEARCH_TARGET}\r\n\r\n"
    )
}

#[test]
fn the_responder_reports_the_port_it_listens_on() {
    let responder = SsdpResponder::bind(
        0,
        "http://10.0.0.5:8001/description.xml".to_string(),
        "uuid:abc".to_string(),
    )
    .expect("the responder binds");
    assert!(responder.local_port() > 0);
    // WHY: the announcement goes to the group and to the port the
    // responder listens on.
    assert_eq!(
        responder.announcement_target(),
        std::net::SocketAddr::from((MULTICAST_ADDRESS, responder.local_port()))
    );
}

#[test]
fn a_search_for_the_multi_screen_service_is_answered() {
    assert!(is_smartview_search(&search()));
}

#[test]
fn the_search_target_is_the_one_samsung_documents() {
    assert_eq!(
        SEARCH_TARGET,
        "urn:samsung.com:service:MultiScreenService:1"
    );
}

#[test]
fn a_search_for_another_service_is_not_answered() {
    let message = search().replace(SEARCH_TARGET, "urn:schemas-upnp-org:device:MediaRenderer:1");
    assert!(!is_smartview_search(&message));
}

#[test]
fn a_message_that_is_not_a_search_is_not_answered() {
    assert!(!is_smartview_search(
        "NOTIFY * HTTP/1.1\r\nNTS: ssdp:alive\r\n\r\n"
    ));
    assert!(!is_smartview_search(""));
    assert!(!is_smartview_search("M-SEARCH"));
}

#[test]
fn a_search_with_a_foreign_man_header_is_not_answered() {
    // WHY: `man` says what the sender wants. A search that announces
    // something else is not a search for an answer.
    let message = search().replace("ssdp:discover", "ssdp:no-discover");
    assert!(!is_smartview_search(&message));
}

#[test]
fn a_search_without_a_man_header_is_answered() {
    // WHY: some Samsung senders omit the header. A receiver that insists on
    // it stays invisible to them.
    let message = search().replace("MAN: \"ssdp:discover\"\r\n", "");
    assert!(is_smartview_search(&message));
}

#[test]
fn the_header_names_are_case_insensitive() {
    let message = search().replace("ST:", "st:");
    assert!(is_smartview_search(&message));
}

#[test]
fn a_search_with_a_broken_length_is_not_answered() {
    // WHY: an `M-SEARCH` is a request without a body. A datagram that
    // announces a length it does not carry is malformed, and a malformed
    // datagram is refused instead of being repaired.
    let message = format!("{}CONTENT-LENGTH: 999999999\r\n\r\n", search());
    assert!(!is_smartview_search(&message));
    // The parser still finds the target of a message that carries a broken
    // length — the answer is refused because of the message, not because
    // the parser was confused by it.
    assert_eq!(requested_target(&message), SEARCH_TARGET);
}

#[test]
fn a_truncated_search_is_not_answered() {
    let truncated = "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN:";
    assert!(!is_smartview_search(truncated));
}

#[test]
fn the_answer_carries_the_location_and_the_target() {
    let response = search_response(
        "http://10.0.0.5:8001/description.xml",
        "uuid:abc",
        SEARCH_TARGET,
    );
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(
        response.contains("LOCATION: http://10.0.0.5:8001/description.xml"),
        "{response}"
    );
    assert!(
        response.contains(&format!("ST: {SEARCH_TARGET}")),
        "{response}"
    );
    assert!(response.contains("USN: uuid:abc::"), "{response}");
}

#[test]
fn the_announcement_announces_the_receiver_and_its_leaving() {
    let alive = notify_alive(
        "http://10.0.0.5:8001/description.xml",
        "uuid:abc",
        SEARCH_TARGET,
    );
    assert!(alive.starts_with("NOTIFY * HTTP/1.1"), "{alive}");
    assert!(alive.contains("NTS: ssdp:alive"), "{alive}");
    assert!(alive.contains(&format!("NT: {SEARCH_TARGET}")), "{alive}");
    assert!(
        alive.contains("LOCATION: http://10.0.0.5:8001/description.xml"),
        "{alive}"
    );

    let byebye = notify_byebye("uuid:abc", SEARCH_TARGET);
    assert!(byebye.contains("NTS: ssdp:byebye"), "{byebye}");
    assert!(!byebye.contains("LOCATION:"), "{byebye}");
}

#[test]
fn a_header_value_is_read_without_its_quotes() {
    let message = "M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\nST: \"urn:samsung.com:service:MultiScreenService:1\"\r\n\r\n";
    assert_eq!(
        header_value(message, "man").as_deref(),
        Some("ssdp:discover")
    );
    assert_eq!(header_value(message, "st").as_deref(), Some(SEARCH_TARGET));
    assert_eq!(header_value(message, "missing"), None);
}

#[test]
fn a_message_without_a_target_has_no_target() {
    assert_eq!(requested_target("M-SEARCH * HTTP/1.1\r\n\r\n"), "");
}

#[test]
fn the_packet_cap_is_the_one_of_the_workspace() {
    // WHY: the cap is owned by the core, so that the browser of the
    // discovery crate and this responder agree.
    assert_eq!(
        MAX_PACKET_BYTES,
        phairplay_core::config::limits::SSDP_MESSAGE_BYTES
    );
    assert_eq!(SSDP_PORT, phairplay_core::config::ports::SSDP);
}
