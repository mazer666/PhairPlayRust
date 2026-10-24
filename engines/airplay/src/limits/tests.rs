use super::*;
use phairplay_core::config::LimitsConfig;

#[test]
fn the_limits_come_from_the_configuration() {
    let mut config = AppConfig::default();
    config.limits.max_header_count = 7;
    config.limits.max_header_bytes = 1234;
    config.limits.max_photo_bytes = 4096;
    config.performance.idle_timeout_ms = 5_000;
    let limits = AirPlayLimits::from_config(&config);
    assert_eq!(limits.max_header_count, 7);
    assert_eq!(limits.max_header_bytes, 1234);
    assert_eq!(limits.max_photo_bytes, 4096);
    assert_eq!(limits.idle_timeout, Duration::from_millis(5_000));
}

#[test]
fn the_message_limit_comes_from_the_protocol_configuration() {
    let mut config = AppConfig::default();
    config.protocols.insert(
        "airplay".to_string(),
        phairplay_core::config::ProtocolConfig {
            enabled: true,
            port: 7000,
            require_pin: false,
            max_message_bytes: 4096,
        },
    );
    let limits = AirPlayLimits::from_config(&config);
    assert_eq!(limits.max_message_bytes, 4096);
    assert_eq!(limits.max_sdp_bytes, 4096);
}

#[test]
fn an_rtp_frame_can_never_exceed_the_message_limit() {
    let mut config = AppConfig::default();
    config.protocols.insert(
        "airplay".to_string(),
        phairplay_core::config::ProtocolConfig {
            enabled: true,
            port: 7000,
            require_pin: false,
            max_message_bytes: 16 * 1024,
        },
    );
    let limits = AirPlayLimits::from_config(&config);
    assert_eq!(limits.max_rtp_frame_bytes, 16 * 1024);
}

#[test]
fn a_nonsensical_limit_is_raised_to_a_usable_floor() {
    // A zero would make every request fail; the engine clamps instead of
    // bricking the receiver.
    let config = AppConfig {
        limits: LimitsConfig {
            max_header_count: 0,
            max_header_bytes: 0,
            ..LimitsConfig::default()
        },
        ..AppConfig::default()
    };
    let limits = AirPlayLimits::from_config(&config);
    assert_eq!(limits.max_header_count, 1);
    assert_eq!(limits.max_header_bytes, 256);
}

#[test]
fn the_default_limits_are_usable() {
    let limits = AirPlayLimits::default();
    assert!(limits.max_message_bytes >= 1024);
    assert!(limits.idle_timeout > Duration::ZERO);
}
