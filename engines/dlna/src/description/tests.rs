#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

#[test]
fn the_device_description_names_the_media_renderer() {
    let document = device_description(
        "Living Room",
        "uuid:abc",
        "http://10.0.0.5:8200/",
        "PhairPlay",
        "Receiver",
    );
    assert!(
        document.contains("<deviceType>urn:schemas-upnp-org:device:MediaRenderer:1</deviceType>")
    );
    assert!(document.contains("<friendlyName>Living Room</friendlyName>"));
    assert!(document.contains("<UDN>uuid:abc</UDN>"));
}

#[test]
fn the_device_description_lists_the_three_services() {
    let document = device_description(
        "Living Room",
        "uuid:abc",
        "http://10.0.0.5:8200/",
        "PhairPlay",
        "Receiver",
    );
    for name in [
        SERVICE_CONNECTION_MANAGER,
        SERVICE_AV_TRANSPORT,
        SERVICE_RENDERING_CONTROL,
    ] {
        assert!(document.contains(&format!("<controlURL>/upnp/control/{name}</controlURL>")));
        assert!(document.contains(&format!("<SCPDURL>/{name}.xml</SCPDURL>")));
        assert!(document.contains(&format!("<eventSubURL>/upnp/event/{name}</eventSubURL>")));
    }
}

#[test]
fn the_device_description_escapes_the_friendly_name() {
    let document = device_description(
        "Ben & Jerry's <TV>",
        "uuid:abc",
        "http://10.0.0.5:8200/",
        "PhairPlay",
        "Receiver",
    );
    assert!(document.contains("Ben &amp; Jerry&apos;s &lt;TV&gt;"));
    assert!(!document.contains("<TV>"));
}

#[test]
fn the_device_description_escapes_the_udn() {
    let document = device_description("Room", "uuid:<bad>", "http://x/", "PhairPlay", "Receiver");
    assert!(document.contains("uuid:&lt;bad&gt;"));
}

#[test]
fn the_service_description_lists_the_actions() {
    let scpd = service_description(SERVICE_AV_TRANSPORT);
    for action in advertised_actions(SERVICE_AV_TRANSPORT) {
        assert!(scpd.contains(&format!("<name>{action}</name>")), "{action}");
    }
}

#[test]
fn the_service_description_declares_the_state_variables() {
    let scpd = service_description(SERVICE_RENDERING_CONTROL);
    assert!(scpd.contains("<name>Volume</name>"));
    assert!(scpd.contains("<name>Mute</name>"));
    assert!(scpd.contains("sendEvents=\"yes\""));
}

#[test]
fn the_transport_state_declares_its_values() {
    let scpd = service_description(SERVICE_AV_TRANSPORT);
    assert!(scpd.contains("<allowedValue>PLAYING</allowedValue>"));
    assert!(scpd.contains("<allowedValue>STOPPED</allowedValue>"));
}

#[test]
fn an_unknown_service_has_no_description() {
    assert!(service_description("Telepathy").is_empty());
    assert!(advertised_actions("Telepathy").is_empty());
}

#[test]
fn the_service_names_are_the_three_upnp_services() {
    assert_eq!(
        service_names(),
        vec![
            SERVICE_CONNECTION_MANAGER,
            SERVICE_AV_TRANSPORT,
            SERVICE_RENDERING_CONTROL
        ]
    );
}

#[test]
fn the_escaping_covers_every_dangerous_character() {
    assert_eq!(escape_xml("<>&\"'"), "&lt;&gt;&amp;&quot;&apos;");
    assert_eq!(escape_xml("plain"), "plain");
}
