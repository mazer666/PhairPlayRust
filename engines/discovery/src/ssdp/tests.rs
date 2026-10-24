#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

#[test]
fn the_query_carries_the_mandatory_headers() {
    let message = SsdpSearchQuery::new(ST_ALL).with_mx(3).to_message();
    assert!(message.starts_with("M-SEARCH * HTTP/1.1\r\n"));
    assert!(message.contains("MAN: \"ssdp:discover\""));
    assert!(message.contains("MX: 3"));
    assert!(message.contains("ST: ssdp:all"));
    assert!(message.ends_with("\r\n\r\n"));
}

#[test]
fn a_response_is_parsed() {
    let message = "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=1800\r\nEXT:\r\n\
                   LOCATION: http://192.168.1.5:8200/description.xml\r\n\
                   SERVER: Linux/1.0 UPnP/1.0\r\nST: ssdp:all\r\n\
                   USN: uuid:abc::upnp:rootdevice\r\n\r\n";
    let response = SsdpResponse::parse(message).expect("a valid response");
    assert_eq!(response.location, "http://192.168.1.5:8200/description.xml");
    assert_eq!(response.usn, "uuid:abc::upnp:rootdevice");
    assert_eq!(response.search_target, "ssdp:all");
    assert_eq!(response.server, "Linux/1.0 UPnP/1.0");
    assert_eq!(response.cache_control, "max-age=1800");
}

#[test]
fn a_response_with_a_lower_case_header_is_parsed() {
    let message = "HTTP/1.1 200 OK\r\nlocation: http://10.0.0.1/d.xml\r\n\r\n";
    let response = SsdpResponse::parse(message).expect("a valid response");
    assert_eq!(response.location, "http://10.0.0.1/d.xml");
}

#[test]
fn a_message_that_is_not_a_response_is_rejected() {
    assert!(SsdpResponse::parse("NOTIFY * HTTP/1.1\r\n\r\n").is_none());
    assert!(SsdpResponse::parse("HTTP/1.1 500 Error\r\n\r\n").is_none());
    assert!(SsdpResponse::parse("").is_none());
}

#[test]
fn a_header_line_without_a_colon_is_rejected() {
    let message = "HTTP/1.1 200 OK\r\nbroken header\r\n\r\n";
    assert!(SsdpResponse::parse(message).is_none());
}

#[test]
fn an_oversized_message_is_rejected() {
    let message = format!(
        "HTTP/1.1 200 OK\r\nLOCATION: {}\r\n\r\n",
        "x".repeat(MAX_SSDP_MESSAGE_BYTES)
    );
    assert!(SsdpResponse::parse(&message).is_none());
}

#[test]
fn too_many_headers_are_rejected() {
    let mut message = String::from("HTTP/1.1 200 OK\r\n");
    for index in 0..(MAX_SSDP_HEADER_COUNT + 5) {
        message.push_str(&format!("X-{index}: value\r\n"));
    }
    message.push_str("\r\n");
    assert!(SsdpResponse::parse(&message).is_none());
}

#[test]
fn an_over_long_header_line_is_rejected() {
    let message = format!(
        "HTTP/1.1 200 OK\r\nLOCATION: {}\r\n\r\n",
        "x".repeat(MAX_SSDP_HEADER_BYTES + 1)
    );
    assert!(SsdpResponse::parse(&message).is_none());
}

#[test]
fn an_alive_notification_is_parsed() {
    let message = "NOTIFY * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\n\
                   CACHE-CONTROL: max-age=1800\r\nLOCATION: http://10.0.0.2/d.xml\r\n\
                   NT: upnp:rootdevice\r\nNTS: ssdp:alive\r\nUSN: uuid:x::upnp:rootdevice\r\n\r\n";
    let notification = SsdpNotification::parse(message).expect("a valid notification");
    assert!(notification.alive);
    assert_eq!(notification.notification_type, "upnp:rootdevice");
    assert_eq!(notification.usn, "uuid:x::upnp:rootdevice");
    assert_eq!(
        notification.location.as_deref(),
        Some("http://10.0.0.2/d.xml")
    );
}

#[test]
fn a_byebye_notification_is_parsed() {
    let message = "NOTIFY * HTTP/1.1\r\nNT: upnp:rootdevice\r\nNTS: ssdp:byebye\r\n\
                   USN: uuid:x::upnp:rootdevice\r\n\r\n";
    let notification = SsdpNotification::parse(message).expect("a valid notification");
    assert!(!notification.alive);
    assert!(notification.location.is_none());
}

#[test]
fn a_notification_with_an_unknown_nts_is_rejected() {
    let message = "NOTIFY * HTTP/1.1\r\nNT: upnp:rootdevice\r\nNTS: ssdp:other\r\n\r\n";
    assert!(SsdpNotification::parse(message).is_none());
}

#[test]
fn the_event_queue_drops_when_the_byte_budget_is_exhausted() {
    let (sender, _receiver) = sync_channel(8);
    let bytes = AtomicUsize::new(0);
    let dropped = AtomicUsize::new(0);
    let event = SsdpEvent::Response(SsdpResponse {
        location: "http://10.0.0.1/description.xml".to_string(),
        ..SsdpResponse::default()
    });
    push_bounded(&sender, event.clone(), &bytes, &dropped, 1024);
    assert_eq!(bytes.load(Ordering::Relaxed), estimate_size(&event));
    assert_eq!(dropped.load(Ordering::Relaxed), 0);
    // The second event does not fit into the remaining budget.
    let size = estimate_size(&event);
    push_bounded(&sender, event, &bytes, &dropped, size);
    assert_eq!(dropped.load(Ordering::Relaxed), 1);
}

#[test]
fn the_event_queue_drops_when_it_is_full() {
    let (sender, _receiver) = sync_channel(1);
    let bytes = AtomicUsize::new(0);
    let dropped = AtomicUsize::new(0);
    let event = SsdpEvent::Notification(SsdpNotification::default());
    push_bounded(&sender, event.clone(), &bytes, &dropped, 1024 * 1024);
    push_bounded(&sender, event, &bytes, &dropped, 1024 * 1024);
    assert_eq!(dropped.load(Ordering::Relaxed), 1);
}

#[test]
fn the_browser_binds_and_stops_cleanly() {
    let mut browser = SsdpBrowser::with_limits(4, 4096).expect("the browser binds");
    browser
        .search(ST_ALL)
        .expect("the query is sent to the multicast group");
    assert_eq!(browser.queued_bytes(), 0);
    browser.stop();
}

#[test]
fn the_browser_rejects_an_oversized_query() {
    let browser = SsdpBrowser::with_limits(4, 4096).expect("the browser binds");
    let query = SsdpSearchQuery::new(&"x".repeat(MAX_SSDP_MESSAGE_BYTES));
    let error = browser.send(&query).expect_err("the query is too large");
    assert!(matches!(error, SsdpError::Send(_)));
}
