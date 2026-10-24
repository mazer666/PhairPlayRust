#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;
use phairplay_core::identity::DeviceIdentity;

fn config() -> DiscoveryConfig {
    let mut identity = DeviceIdentity::with_name("Living Room");
    identity.device_id = "0f9a7c11-2222-3333-4444-555566667777".to_string();
    DiscoveryConfig::new(identity)
}

#[test]
fn the_airplay_pair_is_registered_together() {
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar.clone());
    service.start(&config()).expect("start succeeds");
    let types: Vec<String> = registrar
        .registered()
        .into_iter()
        .map(|s| s.service_type)
        .collect();
    assert!(types.contains(&AIRPLAY_TCP.to_string()));
    assert!(types.contains(&RAOP_TCP.to_string()));
}

#[test]
fn all_four_service_types_are_advertised_by_default() {
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar.clone());
    service.start(&config()).expect("start succeeds");
    assert_eq!(registrar.registered().len(), 4);
}

#[test]
fn the_raop_service_uses_the_mac_prefixed_name() {
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar.clone());
    service.start(&config()).expect("start succeeds");
    let raop = registrar
        .registered()
        .into_iter()
        .find(|s| s.service_type == RAOP_TCP)
        .expect("the raop service is registered");
    assert!(raop.instance_name.starts_with("0F9A7C112222@"));
    assert!(raop.instance_name.contains("Living Room"));
}

#[test]
fn the_instance_name_is_sanitised() {
    let mut config = config();
    config.identity.name = "Living\u{0} Room\n".to_string();
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar.clone());
    service.start(&config).expect("start succeeds");
    let airplay = registrar
        .registered()
        .into_iter()
        .find(|s| s.service_type == AIRPLAY_TCP)
        .expect("the airplay service is registered");
    assert_eq!(airplay.instance_name, "Living Room");
}

#[test]
fn a_failing_registration_withdraws_the_previous_ones() {
    let registrar = Arc::new(RecordingRegistrar::failing_on(RAOP_TCP));
    let service = DiscoveryService::new(registrar.clone());
    let error = service
        .start(&config())
        .expect_err("the raop registration fails");
    assert!(matches!(error, EngineError::Start { .. }));
    // The airplay service was registered before the failure and must
    // have been withdrawn again.
    assert_eq!(registrar.unregistered().len(), 1);
    assert!(!service.status().running);
    assert!(service.status().advertised.is_empty());
    assert!(service.status().last_error.is_some());
}

#[test]
fn starting_twice_is_refused() {
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar);
    service.start(&config()).expect("the first start succeeds");
    let error = service
        .start(&config())
        .expect_err("the second start is refused");
    assert!(matches!(error, EngineError::AlreadyRunning { .. }));
}

#[test]
fn stopping_withdraws_every_service() {
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar.clone());
    service.start(&config()).expect("start succeeds");
    service.stop().expect("stop succeeds");
    assert_eq!(registrar.unregistered().len(), 4);
    assert!(!service.status().running);
}

#[test]
fn stopping_a_service_that_never_started_is_refused() {
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar);
    let error = service.stop().expect_err("stop is refused");
    assert!(matches!(error, EngineError::NotRunning { .. }));
}

#[test]
fn the_status_lists_the_advertised_services() {
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar);
    service.start(&config()).expect("start succeeds");
    let status = service.status();
    assert!(status.running);
    assert_eq!(status.advertised.len(), 4);
    assert!(
        status
            .advertised
            .iter()
            .all(|s| s.full_name.contains("local."))
    );
}

#[test]
fn the_service_selection_can_be_restricted() {
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar.clone());
    let config = config().with_service_types(vec![GOOGLECAST_TCP.to_string()]);
    service.start(&config).expect("start succeeds");
    let registered = registrar.registered();
    assert_eq!(registered.len(), 1);
    assert_eq!(registered[0].service_type, GOOGLECAST_TCP);
}

#[test]
fn an_unknown_service_type_is_ignored() {
    let registrar = Arc::new(RecordingRegistrar::new());
    let service = DiscoveryService::new(registrar.clone());
    let config = config().with_service_types(vec!["_printer._tcp".to_string()]);
    service.start(&config).expect("start succeeds");
    assert!(registrar.registered().is_empty());
}

#[test]
fn the_airplay_service_advertises_the_reference_txt_records() {
    let spec = config()
        .specs()
        .into_iter()
        .find(|s| s.service_type == AIRPLAY_TCP)
        .expect("the airplay spec exists");
    assert_eq!(spec.property("model"), Some("AppleTV5,3"));
    assert_eq!(spec.property("features"), Some("0x5A7FFFF7,0x1E"));
    assert_eq!(spec.port, DEFAULT_AIRPLAY_PORT);
}

#[test]
fn the_cast_service_advertises_the_cast_port() {
    let spec = config()
        .specs()
        .into_iter()
        .find(|s| s.service_type == GOOGLECAST_TCP)
        .expect("the cast spec exists");
    assert_eq!(spec.port, DEFAULT_CAST_PORT);
    assert_eq!(spec.property("bs"), Some("GOOG"));
}

#[test]
fn the_miracast_service_advertises_the_wfd_port() {
    let spec = config()
        .specs()
        .into_iter()
        .find(|s| s.service_type == MIRACAST_TCP)
        .expect("the miracast spec exists");
    assert_eq!(spec.port, DEFAULT_MIRACAST_PORT);
}
