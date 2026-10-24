//! The roundtrip test of the bridge.
//!
//! WHAT: The proof that the bridge works. A value goes *into* the bridge,
//! through it, and comes *out* again — unchanged. The values are the
//! configuration, the identity, the session snapshot, the counters and the
//! errors, because those are the things the native shells depend on.
//!
//! WHY: A bridge that compiles is not a bridge that works. The conversion
//! of a value into its DTO and back is where the bugs live: a field that is
//! forgotten, a number that is truncated, an option that becomes `None`.
//! Only a test that puts a value in and takes it out again can find them.
//!
//! EXAMPLE: the test `the_configuration_survives_the_bridge` writes a
//! configuration with a value in *every* field, reads it back through the
//! facade of the bridge and compares the two.

mod common;

use std::time::Duration;

use phairplay_core::config::AppConfig;
use phairplay_core::protocol::ProtocolKind;
use phairplay_ffi::dto::{
    FfiAppConfig, FfiProtocolKind, FfiProtocolState, FfiReceiverEvent, FfiStreamStats,
};
use phairplay_ffi::error::FfiErrorCode;
use phairplay_ffi::receiver::Receiver;

use common::{
    EventRecorder, FakeVideoDecoder, Must, MustErr, TestEngine, as_protocol_engine, video_sink,
    wait_for_event,
};

/// A configuration with a value in every field.
///
/// WHY: A roundtrip test with default values proves nothing — the defaults
/// are the same on both sides even when a field is never copied. Every field
/// therefore carries a value that is *not* the default.
fn a_configuration_with_every_field_set() -> AppConfig {
    let mut config = AppConfig::default();
    config.device.name = "Attic Receiver".to_string();
    config.device.model = "PhairPlay Rust Receiver".to_string();
    config.device.manufacturer = "PhairPlayRust".to_string();
    config.device.protocol_version = "2.5".to_string();
    config.protocols.insert(
        "airplay".to_string(),
        phairplay_core::config::ProtocolConfig {
            enabled: false,
            port: 7000,
            require_pin: true,
            max_message_bytes: 4096,
        },
    );
    config.protocols.insert(
        "cast".to_string(),
        phairplay_core::config::ProtocolConfig {
            enabled: true,
            port: 8009,
            require_pin: false,
            max_message_bytes: 65536,
        },
    );
    config.limits.video_queue_frames = 42;
    config.limits.video_queue_bytes = 12_345_678;
    config.limits.audio_queue_packets = 7;
    config.limits.max_pairing_attempts = 3;
    config.limits.max_artwork_bytes = 111;
    config.limits.max_photo_bytes = 222;
    config.limits.max_media_bytes = 333;
    config.limits.max_header_count = 44;
    config.limits.max_header_bytes = 555;
    config.performance.first_frame_latency_ms = 66;
    config.performance.minimum_fps = 45;
    config.performance.max_memory_mb = 123;
    config.performance.max_cpu_percent = 55;
    config.performance.idle_timeout_ms = 9999;
    config
}

#[test]
fn the_configuration_survives_the_bridge() {
    // Wert rein …
    let config = a_configuration_with_every_field_set();
    let dto = FfiAppConfig::from(&config);
    let receiver = Receiver::new(dto.clone()).must("the receiver must accept the configuration");

    // … durch die Brücke …
    let read_back = receiver.config();

    // … und Wert wieder raus, identisch.
    assert_eq!(read_back, dto);
    assert_eq!(read_back.device.name, "Attic Receiver");
    assert_eq!(read_back.device.protocol_version, "2.5");
    assert_eq!(read_back.protocols.len(), 2);
    assert_eq!(read_back.limits.video_queue_frames, 42);
    assert_eq!(read_back.limits.max_header_bytes, 555);
    assert_eq!(read_back.performance.idle_timeout_ms, 9999);
    assert_eq!(read_back.performance.minimum_fps, 45);
}

#[test]
fn a_configuration_that_is_written_back_survives_the_bridge() {
    let receiver =
        Receiver::new(FfiAppConfig::from(&AppConfig::default())).must("the receiver must start");
    let config = a_configuration_with_every_field_set();
    let dto = FfiAppConfig::from(&config);

    receiver
        .set_config(dto.clone())
        .must("the configuration must be accepted");

    assert_eq!(receiver.config(), dto);
    assert_eq!(receiver.core().config(), config);
}

#[test]
fn the_identity_survives_the_bridge() {
    let mut config = AppConfig::default();
    config.device.name = "Living Room".to_string();
    let receiver = Receiver::new(FfiAppConfig::from(&config)).must("the receiver must start");

    let identity = receiver.identity();

    assert_eq!(identity.name, "Living Room");
    assert_eq!(identity.model, config.device.model);
    assert_eq!(identity.manufacturer, config.device.manufacturer);
    assert_eq!(identity.protocol_version, config.device.protocol_version);
    assert_eq!(identity.service_instance_name, "Living Room");
    assert!(!identity.device_id.is_empty());
}

#[test]
fn a_session_survives_the_bridge() {
    let recorder = EventRecorder::new();
    let receiver =
        Receiver::new(FfiAppConfig::from(&AppConfig::default())).must("the receiver must start");
    receiver.set_event_sink(recorder.sink());

    let engine = TestEngine::new(ProtocolKind::AirPlay);
    receiver.register_engine(as_protocol_engine(&engine));
    receiver
        .start_protocol("airplay".to_string())
        .must("the engine must start");

    let event = wait_for_event(
        &recorder,
        |event| matches!(event, FfiReceiverEvent::SessionStarted { .. }),
        Duration::from_secs(5),
    );
    let started = match event {
        FfiReceiverEvent::SessionStarted { session } => session,
        other => panic!("unexpected event: {other:?}"),
    };

    let reported = engine
        .sessions()
        .first()
        .cloned()
        .must("the engine must have reported a session");
    assert_eq!(started.id, reported.id.0);
    assert_eq!(started.protocol, FfiProtocolKind::AirPlay);
    assert_eq!(started.sender_name.as_deref(), Some("Test Sender"));
    assert_eq!(started.sender_address.as_deref(), Some("192.168.1.50"));
    assert_eq!(started.video, None);
    assert_eq!(started.audio, None);

    // The registry of the core knows the session, too.
    let sessions = receiver.core().sessions();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, reported.id);
}

#[test]
fn the_counters_survive_the_bridge() {
    let decoder = FakeVideoDecoder::new();
    let receiver =
        Receiver::new(FfiAppConfig::from(&AppConfig::default())).must("the receiver must start");
    receiver.set_video_sink(video_sink(&decoder));

    let engine = TestEngine::new(ProtocolKind::AirPlay);
    receiver.register_engine(as_protocol_engine(&engine));
    receiver
        .start_protocol("airplay".to_string())
        .must("the engine must start");

    engine
        .submit_video_frame(vec![1, 2, 3, 4, 5])
        .must("the frame must reach the decoder");
    engine
        .submit_video_frame(vec![6, 7, 8])
        .must("the frame must reach the decoder");

    let stats: FfiStreamStats = receiver
        .stats("airplay".to_string())
        .must("the counters must be readable");

    assert_eq!(stats.video_frames, 2);
    assert_eq!(stats.video_frames_dropped, 0);
    assert_eq!(stats.bytes_received, 8);
    assert_eq!(stats.drop_ratio, 0.0);
    assert_eq!(decoder.frames().len(), 2);
    assert_eq!(receiver.core().stats(ProtocolKind::AirPlay).video_frames, 2);
}

#[test]
fn the_status_survives_the_bridge() {
    let receiver =
        Receiver::new(FfiAppConfig::from(&AppConfig::default())).must("the receiver must start");
    let engine = TestEngine::new(ProtocolKind::Cast);
    receiver.register_engine(as_protocol_engine(&engine));

    let status = receiver
        .protocol_status("cast".to_string())
        .must("the status must be readable");

    assert_eq!(status.protocol, FfiProtocolKind::Cast);
    assert_eq!(status.state, FfiProtocolState::Idle);
    assert_eq!(status.tcp_port, Some(7000));
    assert!(status.enabled);

    receiver
        .start_protocol("cast".to_string())
        .must("the engine must start");
    let status = receiver
        .protocol_status("cast".to_string())
        .must("the status must be readable");
    assert_eq!(status.state, FfiProtocolState::Advertising);
}

#[test]
fn a_failing_start_reports_the_stable_code() {
    let receiver =
        Receiver::new(FfiAppConfig::from(&AppConfig::default())).must("the receiver must start");
    let engine = TestEngine::failing(ProtocolKind::Miracast);
    receiver.register_engine(as_protocol_engine(&engine));

    let error = receiver
        .start_protocol("miracast".to_string())
        .must_err("the engine must refuse to start");

    // WHY: the number comes from the core and is part of the contract.
    assert_eq!(error.code(), FfiErrorCode::Internal.as_i32());
    assert!(
        error.details().contains("the port is already in use"),
        "{error}"
    );
}

#[test]
fn an_unknown_protocol_reports_the_stable_code() {
    let receiver =
        Receiver::new(FfiAppConfig::from(&AppConfig::default())).must("the receiver must start");

    for name in ["", "   ", "telepathy", "air play", "dlna/../etc"] {
        let error = receiver
            .protocol_status(name.to_string())
            .must_err("the protocol must be rejected");
        assert_eq!(
            error.code(),
            FfiErrorCode::InvalidInput.as_i32(),
            "the name `{name}` must be rejected with the code of the core"
        );
    }
}

#[test]
fn the_bridge_never_panics_on_hostile_input() {
    // WHY: the bridge is the border to an application that cannot survive a
    // crash of the library. Every call that a host can reach must answer
    // with a structured failure, never with a panic.
    let receiver =
        Receiver::new(FfiAppConfig::from(&AppConfig::default())).must("the receiver must start");

    let hostile_names = [
        String::new(),
        "\u{0}".to_string(),
        "a".repeat(4096),
        "../../etc/passwd".to_string(),
        "airplay\u{0}miracast".to_string(),
    ];

    for name in hostile_names {
        let _ = receiver.protocol_status(name.clone());
        let _ = receiver.start_protocol(name.clone());
        let _ = receiver.stop_protocol(name.clone());
        let _ = receiver.stats(name);
    }

    let _ = receiver.status();
    let _ = receiver.sessions();
    let _ = receiver.config();
    let _ = receiver.identity();
    let _ = receiver.capabilities();
    let _ = receiver.build_info();
}

#[test]
fn the_bridge_reports_an_invalid_configuration_instead_of_accepting_it() {
    let mut config = FfiAppConfig::from(&AppConfig::default());
    config.limits.max_pairing_attempts = 0;
    let error = Receiver::new(config).must_err("the configuration must be rejected");
    assert_eq!(error.code(), FfiErrorCode::InvalidInput.as_i32());
    assert!(error.details().contains("max_pairing_attempts"), "{error}");
}

#[test]
fn the_bridge_reports_a_configuration_with_an_unknown_protocol() {
    let mut config = FfiAppConfig::from(&AppConfig::default());
    config
        .protocols
        .push(phairplay_ffi::dto::FfiProtocolConfig {
            name: "telepathy".to_string(),
            enabled: true,
            port: 0,
            require_pin: false,
            max_message_bytes: 1024,
        });
    let error = Receiver::new(config).must_err("the configuration must be rejected");
    assert_eq!(error.code(), FfiErrorCode::InvalidInput.as_i32());
    assert!(error.details().contains("telepathy"), "{error}");
}
