//! The end-to-end tests of the discovery of the Smart View receiver.
//!
//! WHY: the SSDP codec can be green while the receiver is invisible: a
//! socket that is bound to the wrong port, a datagram that never leaves the
//! host or a search that is answered with the wrong header. The tests
//! therefore send real datagrams over a real socket and read the answer
//! that comes back.
//!
//! HOW: the responder of the test binds a free port, so that the test does
//! not fight with the DLNA engine of the same host over the SSDP port. The
//! search is sent unicast to that port — the sender of a real search
//! reaches the responder the same way.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::net::UdpSocket;
use std::time::Duration;

use phairplay_smartview::ssdp::{MULTICAST_ADDRESS, SEARCH_TARGET, SSDP_PORT, SsdpResponder};
use socket2::{Domain, Protocol, Socket, Type};

/// The location the responder announces.
const LOCATION: &str = "http://127.0.0.1:8001/description.xml";

/// The unique device name the responder announces.
const UDN: &str = "uuid:abc";

/// A search of a Galaxy device.
fn search() -> String {
    format!(
        "M-SEARCH * HTTP/1.1\r\nHOST: {MULTICAST_ADDRESS}:{SSDP_PORT}\r\n\
         MAN: \"ssdp:discover\"\r\nMX: 3\r\nST: {SEARCH_TARGET}\r\n\r\n"
    )
}

/// A socket that sends a datagram and waits for the answer.
fn sender() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("the socket binds");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("the timeout is set");
    socket
}

/// Reads an answer, or returns `None` when none arrives.
fn read_answer(socket: &UdpSocket) -> Option<String> {
    let mut buffer = vec![0u8; 2048];
    match socket.recv_from(&mut buffer) {
        Ok((read, _)) => Some(String::from_utf8_lossy(&buffer[..read]).to_string()),
        Err(_) => None,
    }
}

#[test]
fn a_search_for_the_multi_screen_service_is_answered_over_the_network() {
    let responder = SsdpResponder::bind(0).expect("the responder binds");
    responder.add_service(phairplay_smartview::ssdp::service(LOCATION, UDN));
    let socket = sender();
    socket
        .send_to(search().as_bytes(), ("127.0.0.1", responder.local_port()))
        .expect("the search is sent");
    let answer = read_answer(&socket).expect("the answer arrives");
    assert!(answer.starts_with("HTTP/1.1 200 OK"), "{answer}");
    assert!(
        answer.contains(&format!("LOCATION: {LOCATION}")),
        "{answer}"
    );
    assert!(answer.contains(&format!("ST: {SEARCH_TARGET}")), "{answer}");
    assert!(answer.contains("USN: uuid:abc::"), "{answer}");
}

#[test]
fn a_search_with_a_broken_length_is_not_answered_and_the_responder_survives() {
    // WHY: a datagram that announces a length it does not carry is
    // malformed. The responder has to refuse it — and it has to stay alive:
    // a parser that panics on a broken datagram takes the whole
    // discovery of the receiver down.
    let responder = SsdpResponder::bind(0).expect("the responder binds");
    responder.add_service(phairplay_smartview::ssdp::service(LOCATION, UDN));
    let socket = sender();
    let hostile = format!("{}CONTENT-LENGTH: 999999999\r\n\r\n", search());
    socket
        .send_to(hostile.as_bytes(), ("127.0.0.1", responder.local_port()))
        .expect("the broken search is sent");
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .expect("the timeout is set");
    assert!(
        read_answer(&socket).is_none(),
        "the broken search must not be answered"
    );
    // The responder is still alive and answers a well-formed search.
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("the timeout is set");
    socket
        .send_to(search().as_bytes(), ("127.0.0.1", responder.local_port()))
        .expect("the search is sent");
    let answer = read_answer(&socket).expect("the answer arrives");
    assert!(answer.starts_with("HTTP/1.1 200 OK"), "{answer}");
}

#[test]
fn a_search_for_another_service_is_not_answered() {
    let responder = SsdpResponder::bind(0).expect("the responder binds");
    responder.add_service(phairplay_smartview::ssdp::service(LOCATION, UDN));
    let socket = sender();
    let hostile = search().replace(SEARCH_TARGET, "urn:schemas-upnp-org:device:MediaRenderer:1");
    socket
        .send_to(hostile.as_bytes(), ("127.0.0.1", responder.local_port()))
        .expect("the search is sent");
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .expect("the timeout is set");
    assert!(
        read_answer(&socket).is_none(),
        "a search for another service must not be answered"
    );
}

#[test]
fn the_announcement_reaches_the_multicast_group() {
    // WHY: the announcement is what makes the device appear without a
    // search. It has to reach the group and the port the responder listens
    // on — in the configuration of a receiver that is the well-known SSDP
    // port 1900.
    //
    // The test binds a free port instead of the well-known one, because the
    // DLNA engine of the same host may own the port 1900 — and because the
    // property under test is "the announcement reaches the group", not "the
    // port is free".
    let listener =
        Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("the socket exists");
    listener
        .set_reuse_address(true)
        .expect("the address may be reused");
    listener
        .bind(&std::net::SocketAddr::from(([0, 0, 0, 0], 0)).into())
        .expect("the socket binds");
    listener
        .join_multicast_v4(&MULTICAST_ADDRESS, &std::net::Ipv4Addr::UNSPECIFIED)
        .expect("the group is joined");
    let listener: UdpSocket = listener.into();
    let port = listener
        .local_addr()
        .expect("the socket has an address")
        .port();
    listener
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("the timeout is set");

    let responder = SsdpResponder::bind(port).expect("the responder binds");
    responder.add_service(phairplay_smartview::ssdp::service(LOCATION, UDN));
    assert_eq!(
        responder.announcement_target(),
        std::net::SocketAddr::from((MULTICAST_ADDRESS, port))
    );
    let service = phairplay_smartview::ssdp::service(LOCATION, UDN);
    responder
        .announce(&service)
        .expect("the announcement is sent");

    let mut buffer = vec![0u8; 2048];
    let (read, _) = listener
        .recv_from(&mut buffer)
        .expect("the announcement arrives");
    let announcement = String::from_utf8_lossy(&buffer[..read]).to_string();
    assert!(
        announcement.starts_with("NOTIFY * HTTP/1.1"),
        "{announcement}"
    );
    assert!(announcement.contains("NTS: ssdp:alive"), "{announcement}");
    assert!(
        announcement.contains(&format!("NT: {SEARCH_TARGET}")),
        "{announcement}"
    );
    assert!(
        announcement.contains(&format!("LOCATION: {LOCATION}")),
        "{announcement}"
    );
}

#[test]
fn the_announcement_of_a_receiver_uses_the_well_known_ssdp_port() {
    // WHY: the port 1900 is the contract with the sender. The default
    // configuration of the engine has to announce there.
    assert_eq!(SSDP_PORT, 1900);
    assert_eq!(
        MULTICAST_ADDRESS,
        std::net::Ipv4Addr::new(239, 255, 255, 250)
    );
}
