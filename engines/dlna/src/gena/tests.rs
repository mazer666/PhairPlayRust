#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

fn manager() -> GenaEventManager {
    GenaEventManager::new(Ipv4Addr::new(192, 168, 1, 5))
}

#[test]
fn a_control_point_on_the_same_subnet_may_subscribe() {
    let manager = manager();
    let subscription = manager
        .subscribe(
            Some("<http://192.168.1.20:8080/events>"),
            "/upnp/event/AVTransport",
        )
        .expect("the subscription is accepted");
    assert!(subscription.sid.starts_with("uuid:"));
    assert_eq!(subscription.timeout_seconds, SUBSCRIPTION_TIMEOUT_SECONDS);
    assert_eq!(manager.subscription_count(), 1);
}

#[test]
fn the_cloud_metadata_address_is_refused() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("http://169.254.169.254/latest/meta-data/"));
    let error = manager
        .subscribe(
            Some("<http://169.254.169.254/latest/meta-data/>"),
            "/upnp/event/AVTransport",
        )
        .expect_err("the subscription is refused");
    assert!(matches!(error, GenaError::ForbiddenCallback(_)));
    assert_eq!(manager.subscription_count(), 0);
}

#[test]
fn the_loopback_address_is_refused() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("http://127.0.0.1:8080/events"));
    assert!(!manager.is_allowed_callback("http://127.0.0.1:8080/"));
}

#[test]
fn the_ipv6_loopback_is_refused() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("http://[::1]:8080/events"));
    assert!(!manager.is_allowed_callback("http://::1/events"));
}

#[test]
fn an_address_outside_the_local_subnet_is_refused() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("http://192.168.2.20:8080/events"));
    assert!(!manager.is_allowed_callback("http://10.0.0.5:8080/events"));
}

#[test]
fn a_public_address_is_refused() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("http://93.184.216.34/events"));
    assert!(!manager.is_allowed_callback("http://8.8.8.8/events"));
}

#[test]
fn a_host_name_is_refused_because_it_is_never_resolved() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("http://metadata.google.internal/"));
    assert!(!manager.is_allowed_callback("http://localhost:8080/events"));
    assert!(!manager.is_allowed_callback("http://evil.example.com/events"));
}

#[test]
fn a_credential_in_the_url_is_refused() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("http://192.168.1.20@169.254.169.254/"));
}

#[test]
fn a_non_http_scheme_is_refused() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("file:///etc/passwd"));
    assert!(!manager.is_allowed_callback("gopher://192.168.1.20/"));
    assert!(!manager.is_allowed_callback("https://192.168.1.20/"));
}

#[test]
fn a_url_without_a_scheme_is_refused() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("192.168.1.20:8080/events"));
}

#[test]
fn a_url_with_a_fragment_is_refused() {
    let manager = manager();
    assert!(!manager.is_allowed_callback("http://192.168.1.20/events#x"));
}

#[test]
fn a_callback_without_angle_brackets_is_refused() {
    let manager = manager();
    let error = manager
        .subscribe(
            Some("http://192.168.1.20:8080/events"),
            "/upnp/event/AVTransport",
        )
        .expect_err("the subscription is refused");
    assert!(matches!(error, GenaError::MalformedCallback(_)));
}

#[test]
fn a_missing_callback_is_refused() {
    let manager = manager();
    let error = manager
        .subscribe(None, "/upnp/event/AVTransport")
        .expect_err("the subscription is refused");
    assert_eq!(error, GenaError::MissingCallback);
}

#[test]
fn a_subscription_can_be_removed() {
    let manager = manager();
    let subscription = manager
        .subscribe(
            Some("<http://192.168.1.20:8080/events>"),
            "/upnp/event/AVTransport",
        )
        .expect("accepted");
    manager
        .unsubscribe(Some(&subscription.sid))
        .expect("the subscription is removed");
    assert_eq!(manager.subscription_count(), 0);
    assert_eq!(
        manager.unsubscribe(Some(&subscription.sid)),
        Err(GenaError::UnknownSid)
    );
}

#[test]
fn the_first_of_several_callbacks_is_used() {
    let manager = manager();
    manager
        .subscribe(
            Some("<http://192.168.1.20:8080/a> <http://192.168.1.21:8080/b>"),
            "/upnp/event/AVTransport",
        )
        .expect("accepted");
    assert_eq!(manager.subscription_count(), 1);
}

#[test]
fn the_private_ranges_are_recognised() {
    assert!(is_private_ipv4(Ipv4Addr::new(10, 0, 0, 1)));
    assert!(is_private_ipv4(Ipv4Addr::new(172, 16, 0, 1)));
    assert!(is_private_ipv4(Ipv4Addr::new(172, 31, 255, 255)));
    assert!(is_private_ipv4(Ipv4Addr::new(192, 168, 1, 1)));
    assert!(!is_private_ipv4(Ipv4Addr::new(172, 32, 0, 1)));
    assert!(!is_private_ipv4(Ipv4Addr::new(169, 254, 169, 254)));
    assert!(!is_private_ipv4(Ipv4Addr::new(127, 0, 0, 1)));
}

#[test]
fn the_subnet_comparison_uses_three_octets() {
    assert!(same_subnet(
        Ipv4Addr::new(192, 168, 1, 5),
        Ipv4Addr::new(192, 168, 1, 200)
    ));
    assert!(!same_subnet(
        Ipv4Addr::new(192, 168, 1, 5),
        Ipv4Addr::new(192, 168, 2, 5)
    ));
}

#[test]
fn the_url_parser_rejects_malformed_urls() {
    assert!(parse_http_url("").is_err());
    assert!(parse_http_url("http://").is_err());
    assert!(parse_http_url("http://host:0/").is_err());
    assert!(parse_http_url("http://host:99999/").is_err());
    assert!(parse_http_url("http://[::1]/").is_err());
}

#[test]
fn the_url_parser_reads_the_port_and_the_path() {
    let url = parse_http_url("http://192.168.1.20:8080/events?x=1").expect("valid");
    assert_eq!(url.scheme, "http");
    assert_eq!(url.host, "192.168.1.20");
    assert_eq!(url.port, Some(8080));
    assert_eq!(url.path, "/events?x=1");
}

#[test]
fn a_url_without_a_path_gets_the_root() {
    let url = parse_http_url("http://192.168.1.20").expect("valid");
    assert_eq!(url.path, "/");
    assert_eq!(url.port, None);
}

#[test]
fn the_subscription_identifiers_are_unique() {
    let manager = manager();
    let first = manager
        .subscribe(Some("<http://192.168.1.20:8080/a>"), "/s")
        .expect("accepted");
    let second = manager
        .subscribe(Some("<http://192.168.1.20:8080/b>"), "/s")
        .expect("accepted");
    assert_ne!(first.sid, second.sid);
}
