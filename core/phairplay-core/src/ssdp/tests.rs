#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

fn service(target: &str, usn: &str) -> SsdpService {
    SsdpService {
        filter: super::SearchFilter::new(|message| message.starts_with("M-SEARCH ")),
        primary_target: target.to_string(),
        search_targets: vec![target.to_string()],
        notification_targets: vec![target.to_string()],
        location: format!("http://192.168.1.10:8001/{usn}.xml"),
        usn: usn.to_string(),
        server: "PhairPlayRust/1.0".to_string(),
    }
}

#[test]
fn a_search_is_recognised() {
    let message = "M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\nST: ssdp:all\r\n\r\n";
    assert!(is_search(message));
}

#[test]
fn a_notification_is_not_a_search() {
    let message = "NOTIFY * HTTP/1.1\r\nNTS: ssdp:alive\r\n\r\n";
    assert!(!is_search(message));
}

#[test]
fn the_search_target_is_read() {
    let message = "M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\nST: urn:samsung.com:service:MultiScreenService:1\r\n\r\n";
    assert_eq!(
        requested_target(message),
        "urn:samsung.com:service:MultiScreenService:1"
    );
}

#[test]
fn a_query_without_a_target_asks_for_every_device() {
    let message = "M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\n\r\n";
    assert_eq!(requested_target(message), ST_ALL);
}

#[test]
fn a_service_answers_for_its_own_target_only() {
    let dlna = service("urn:schemas-upnp-org:device:MediaRenderer:1", "uuid:a");
    let smartview = service("urn:samsung.com:service:MultiScreenService:1", "uuid:b");
    assert!(dlna.answers("urn:schemas-upnp-org:device:MediaRenderer:1"));
    assert!(!dlna.answers("urn:samsung.com:service:MultiScreenService:1"));
    assert!(smartview.answers("urn:samsung.com:service:MultiScreenService:1"));
    assert!(!smartview.answers("urn:schemas-upnp-org:device:MediaRenderer:1"));
}

#[test]
fn a_service_answers_for_the_wildcard() {
    let dlna = service("urn:schemas-upnp-org:device:MediaRenderer:1", "uuid:a");
    assert!(dlna.answers(ST_ALL));
}

#[test]
fn a_query_for_everything_is_answered_with_the_type_of_the_service() {
    // WHY: the answer has to tell the control point what it found. An
    // answer that carries "ssdp:all" back tells it nothing.
    let dlna = service("urn:schemas-upnp-org:device:MediaRenderer:1", "uuid:a");
    assert_eq!(
        dlna.answer_target(ST_ALL),
        Some("urn:schemas-upnp-org:device:MediaRenderer:1".to_string())
    );
}

#[test]
fn the_answer_carries_the_location_the_target_and_the_unique_name() {
    let answer = search_response(
        "http://192.168.1.10:8001/description.xml",
        "uuid:phairplay",
        "urn:samsung.com:service:MultiScreenService:1",
        "PhairPlayRust/1.0",
    );
    assert!(answer.starts_with("HTTP/1.1 200 OK"), "{answer}");
    assert!(
        answer.contains("LOCATION: http://192.168.1.10:8001/description.xml"),
        "{answer}"
    );
    assert!(
        answer.contains("ST: urn:samsung.com:service:MultiScreenService:1"),
        "{answer}"
    );
    assert!(answer.contains("USN: uuid:phairplay::"), "{answer}");
    assert!(answer.contains("SERVER: PhairPlayRust/1.0"), "{answer}");
}

#[test]
fn the_announcement_carries_the_state_and_the_target() {
    let alive = notify_alive(
        "http://192.168.1.10:8001/description.xml",
        "uuid:phairplay",
        "urn:samsung.com:service:MultiScreenService:1",
        "PhairPlayRust/1.0",
    );
    assert!(alive.starts_with("NOTIFY * HTTP/1.1"), "{alive}");
    assert!(alive.contains("NTS: ssdp:alive"), "{alive}");
    assert!(
        alive.contains("NT: urn:samsung.com:service:MultiScreenService:1"),
        "{alive}"
    );

    let byebye = notify_byebye(
        "uuid:phairplay",
        "urn:samsung.com:service:MultiScreenService:1",
    );
    assert!(byebye.contains("NTS: ssdp:byebye"), "{byebye}");
}

#[test]
fn two_services_share_one_responder() {
    // WHY: this is the reason the responder lives in the core. Linux allows
    // one socket per multicast group, so the two services of the receiver
    // need one listener — and one listener has to answer for both of them.
    let responder = SsdpResponder::bind(0).expect("the responder must bind");
    let dlna = service("urn:schemas-upnp-org:device:MediaRenderer:1", "uuid:a");
    let smartview = service("urn:samsung.com:service:MultiScreenService:1", "uuid:b");
    responder.add_service(dlna.clone());
    responder.add_service(smartview.clone());

    let services = responder.services();
    assert_eq!(services.len(), 2, "{services:?}");

    responder.remove_service("uuid:a");
    assert_eq!(responder.services().len(), 1);
    responder.stop();
}

#[test]
fn a_service_that_is_started_twice_answers_once() {
    let responder = SsdpResponder::bind(0).expect("the responder must bind");
    let dlna = service("urn:schemas-upnp-org:device:MediaRenderer:1", "uuid:a");
    responder.add_service(dlna.clone());
    responder.add_service(dlna);
    assert_eq!(responder.services().len(), 1);
    responder.stop();
}
