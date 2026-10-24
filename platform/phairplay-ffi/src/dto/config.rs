//! The configuration as it travels over the bridge.
//!
//! WHAT: The whole configuration of the receiver — the identity, the
//! per-protocol settings, the limits and the performance budgets — in a
//! shape that a foreign language can carry.
//!
//! WHY: The configuration is the only thing a user can change without
//! touching the source. If every host kept its own copy, the receiver and
//! the dashboard would disagree about which protocols are enabled. The
//! bridge therefore carries the configuration in both directions and never
//! keeps a second copy of it.
//!
//! The protocol list is a `Vec` and not a map, because a map with a string
//! key is awkward to build in Swift and in Kotlin. The name of the protocol
//! is part of the entry.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::FfiAppConfig;
//! use phairplay_core::config::AppConfig;
//!
//! let config = FfiAppConfig::from(&AppConfig::default());
//! assert_eq!(config.device.name, "PhairPlay");
//! ```

use std::collections::BTreeMap;

use phairplay_core::config::{
    AppConfig, DeviceConfig, LimitsConfig, PerformanceConfig, ProtocolConfig,
};

use crate::error::FfiError;

/// The configuration of the receiver.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiAppConfig {
    /// The identity advertised to senders.
    pub device: FfiDeviceConfig,
    /// The configuration of every protocol that is configured.
    pub protocols: Vec<FfiProtocolConfig>,
    /// The global limits.
    pub limits: FfiLimitsConfig,
    /// The performance budgets the implementation is measured against.
    pub performance: FfiPerformanceConfig,
}

impl From<&AppConfig> for FfiAppConfig {
    fn from(config: &AppConfig) -> Self {
        FfiAppConfig {
            device: FfiDeviceConfig::from(&config.device),
            protocols: config
                .protocols
                .iter()
                .map(|(name, protocol)| FfiProtocolConfig {
                    name: name.clone(),
                    enabled: protocol.enabled,
                    port: protocol.port,
                    require_pin: protocol.require_pin,
                    max_message_bytes: protocol.max_message_bytes as u64,
                })
                .collect(),
            limits: FfiLimitsConfig::from(&config.limits),
            performance: FfiPerformanceConfig::from(&config.performance),
        }
    }
}

impl From<AppConfig> for FfiAppConfig {
    fn from(config: AppConfig) -> Self {
        FfiAppConfig::from(&config)
    }
}

impl TryFrom<&FfiAppConfig> for AppConfig {
    type Error = FfiError;

    fn try_from(config: &FfiAppConfig) -> Result<Self, Self::Error> {
        let mut protocols = BTreeMap::new();
        for protocol in &config.protocols {
            if protocols.contains_key(&protocol.name) {
                return Err(FfiError::invalid_input(format!(
                    "the protocol `{}` is configured twice",
                    protocol.name
                )));
            }
            let max_message_bytes = usize::try_from(protocol.max_message_bytes).map_err(|_| {
                FfiError::invalid_input(format!(
                    "the maximum message size of `{}` does not fit into this platform",
                    protocol.name
                ))
            })?;
            protocols.insert(
                protocol.name.clone(),
                ProtocolConfig {
                    enabled: protocol.enabled,
                    port: protocol.port,
                    require_pin: protocol.require_pin,
                    max_message_bytes,
                },
            );
        }
        Ok(AppConfig {
            device: DeviceConfig::from(&config.device),
            protocols,
            limits: LimitsConfig::from(&config.limits),
            performance: PerformanceConfig::from(&config.performance),
        })
    }
}

impl TryFrom<FfiAppConfig> for AppConfig {
    type Error = FfiError;

    fn try_from(config: FfiAppConfig) -> Result<Self, Self::Error> {
        AppConfig::try_from(&config)
    }
}

/// The identity configuration.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiDeviceConfig {
    /// The user-visible device name.
    pub name: String,
    /// The hardware model string.
    pub model: String,
    /// The manufacturer string.
    pub manufacturer: String,
    /// The protocol version reported to senders.
    pub protocol_version: String,
}

impl From<&DeviceConfig> for FfiDeviceConfig {
    fn from(device: &DeviceConfig) -> Self {
        FfiDeviceConfig {
            name: device.name.clone(),
            model: device.model.clone(),
            manufacturer: device.manufacturer.clone(),
            protocol_version: device.protocol_version.clone(),
        }
    }
}

impl From<&FfiDeviceConfig> for DeviceConfig {
    fn from(device: &FfiDeviceConfig) -> Self {
        DeviceConfig {
            name: device.name.clone(),
            model: device.model.clone(),
            manufacturer: device.manufacturer.clone(),
            protocol_version: device.protocol_version.clone(),
        }
    }
}

/// The configuration of one protocol.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiProtocolConfig {
    /// The name of the protocol, for example "airplay".
    pub name: String,
    /// Whether the protocol is enabled.
    pub enabled: bool,
    /// The TCP port the protocol listens on.
    pub port: u16,
    /// Whether a PIN is required before a peer may stream.
    pub require_pin: bool,
    /// The maximum number of bytes accepted in a single protocol message.
    pub max_message_bytes: u64,
}

/// The global resource limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct FfiLimitsConfig {
    /// The maximum number of decoded video frames buffered in the sink queue.
    pub video_queue_frames: u64,
    /// The maximum total number of bytes buffered in the video sink queue.
    pub video_queue_bytes: u64,
    /// The maximum number of audio packets buffered in the sink queue.
    pub audio_queue_packets: u64,
    /// The maximum number of pairing attempts before the sender is locked out.
    pub max_pairing_attempts: u32,
    /// The maximum size of an album artwork payload, in bytes.
    pub max_artwork_bytes: u64,
    /// The maximum size of a photo payload, in bytes.
    pub max_photo_bytes: u64,
    /// The maximum size of a DLNA media payload, in bytes.
    pub max_media_bytes: u64,
    /// The maximum number of headers accepted in one HTTP-like request.
    pub max_header_count: u64,
    /// The maximum length of a single header line, in bytes.
    pub max_header_bytes: u64,
}

impl From<&LimitsConfig> for FfiLimitsConfig {
    fn from(limits: &LimitsConfig) -> Self {
        FfiLimitsConfig {
            video_queue_frames: limits.video_queue_frames as u64,
            video_queue_bytes: limits.video_queue_bytes as u64,
            audio_queue_packets: limits.audio_queue_packets as u64,
            max_pairing_attempts: limits.max_pairing_attempts,
            max_artwork_bytes: limits.max_artwork_bytes as u64,
            max_photo_bytes: limits.max_photo_bytes as u64,
            max_media_bytes: limits.max_media_bytes as u64,
            max_header_count: limits.max_header_count as u64,
            max_header_bytes: limits.max_header_bytes as u64,
        }
    }
}

impl From<&FfiLimitsConfig> for LimitsConfig {
    fn from(limits: &FfiLimitsConfig) -> Self {
        LimitsConfig {
            video_queue_frames: limits.video_queue_frames as usize,
            video_queue_bytes: limits.video_queue_bytes as usize,
            audio_queue_packets: limits.audio_queue_packets as usize,
            max_pairing_attempts: limits.max_pairing_attempts,
            max_artwork_bytes: limits.max_artwork_bytes as usize,
            max_photo_bytes: limits.max_photo_bytes as usize,
            max_media_bytes: limits.max_media_bytes as usize,
            max_header_count: limits.max_header_count as usize,
            max_header_bytes: limits.max_header_bytes as usize,
        }
    }
}

/// The performance budgets of the implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct FfiPerformanceConfig {
    /// The maximum time from accepting a connection to the first decoded frame.
    pub first_frame_latency_ms: u64,
    /// The minimum sustained frame rate during mirroring.
    pub minimum_fps: u32,
    /// The maximum resident memory of the receiver process, in megabytes.
    pub max_memory_mb: u32,
    /// The maximum sustained CPU usage, in percent.
    pub max_cpu_percent: u32,
    /// The maximum time a peer may stay connected without sending anything.
    pub idle_timeout_ms: u64,
}

impl From<&PerformanceConfig> for FfiPerformanceConfig {
    fn from(performance: &PerformanceConfig) -> Self {
        FfiPerformanceConfig {
            first_frame_latency_ms: performance.first_frame_latency_ms,
            minimum_fps: performance.minimum_fps,
            max_memory_mb: performance.max_memory_mb,
            max_cpu_percent: performance.max_cpu_percent,
            idle_timeout_ms: performance.idle_timeout_ms,
        }
    }
}

impl From<&FfiPerformanceConfig> for PerformanceConfig {
    fn from(performance: &FfiPerformanceConfig) -> Self {
        PerformanceConfig {
            first_frame_latency_ms: performance.first_frame_latency_ms,
            minimum_fps: performance.minimum_fps,
            max_memory_mb: performance.max_memory_mb,
            max_cpu_percent: performance.max_cpu_percent,
            idle_timeout_ms: performance.idle_timeout_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{Must, MustErr};

    #[test]
    fn the_configuration_survives_the_conversion() {
        let mut config = AppConfig::default();
        config.device.name = "Living Room".into();
        config.protocols.insert(
            "airplay".into(),
            ProtocolConfig {
                enabled: false,
                port: 7000,
                require_pin: true,
                max_message_bytes: 4096,
            },
        );
        config.limits.video_queue_frames = 30;
        config.performance.minimum_fps = 60;

        let converted = FfiAppConfig::from(&config);
        let back = AppConfig::try_from(&converted).must("the configuration must convert back");
        assert_eq!(back, config);
    }

    #[test]
    fn a_protocol_that_is_configured_twice_is_rejected() {
        let config = FfiAppConfig {
            device: FfiDeviceConfig::from(&DeviceConfig::default()),
            protocols: vec![
                FfiProtocolConfig {
                    name: "airplay".into(),
                    enabled: true,
                    port: 0,
                    require_pin: false,
                    max_message_bytes: 1024,
                },
                FfiProtocolConfig {
                    name: "airplay".into(),
                    enabled: false,
                    port: 0,
                    require_pin: false,
                    max_message_bytes: 1024,
                },
            ],
            limits: FfiLimitsConfig::from(&LimitsConfig::default()),
            performance: FfiPerformanceConfig::from(&PerformanceConfig::default()),
        };
        let error = AppConfig::try_from(&config).must_err("the duplicate must be rejected");
        assert_eq!(
            error.code(),
            crate::error::FfiErrorCode::InvalidInput.as_i32()
        );
    }

    #[test]
    fn the_default_configuration_survives_the_conversion() {
        let config = AppConfig::default();
        let converted = FfiAppConfig::from(&config);
        let back = AppConfig::try_from(&converted).must("the configuration must convert back");
        assert_eq!(back, config);
    }
}
