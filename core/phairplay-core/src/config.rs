//! The configuration model.
//!
//! WHY: defaults, limits and policy must not be scattered through the code.
//! They live in one typed structure, are loaded from one place, and are
//! documented in `config/`.
//!
//! HOW: [`AppConfig`] is deserialised from TOML. Every field has a default, so
//! a missing key is never an error — only a *wrong* value is.
//!
//! ## The modules of this configuration
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`ports`] | The well-known ports of the receiver protocols. |
//! | [`service_types`] | The DNS-SD service types the receiver advertises. |
//! | [`timeouts`] | The protocol-internal deadlines. |
//! | [`limits`] | The size caps that more than one crate has to know. |
//! | [`engine_limits`] | The limits of one protocol, resolved out of [`AppConfig`]. |
//!
//! Every number that is written down more than once in the workspace has its
//! single owner in one of these modules. The engine crates keep their own
//! public names, but the *value* is read from here.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;
use crate::protocol::ProtocolKind;

pub mod engine_limits;
pub mod limits;
pub mod ports;
pub mod service_types;
pub mod timeouts;

pub use engine_limits::EngineLimits;

/// The top-level configuration of the receiver.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    /// The identity advertised to senders.
    pub device: DeviceConfig,
    /// The per-protocol configuration.
    pub protocols: BTreeMap<String, ProtocolConfig>,
    /// The global limits.
    pub limits: LimitsConfig,
    /// The performance budgets the implementation is measured against.
    pub performance: PerformanceConfig,
}

/// The identity configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DeviceConfig {
    /// The user-visible device name.
    pub name: String,
    /// The hardware model string.
    pub model: String,
    /// The manufacturer string.
    pub manufacturer: String,
    /// The protocol version reported to senders.
    pub protocol_version: String,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            name: "PhairPlay".to_string(),
            model: "PhairPlay Rust Receiver".to_string(),
            manufacturer: "PhairPlayRust".to_string(),
            protocol_version: "1.0".to_string(),
        }
    }
}

/// The configuration of a single protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProtocolConfig {
    /// Whether the protocol is enabled.
    pub enabled: bool,
    /// The TCP port the protocol listens on.
    pub port: u16,
    /// Whether a PIN is required before a peer may stream.
    pub require_pin: bool,
    /// The maximum number of bytes accepted in a single protocol message.
    pub max_message_bytes: usize,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 0,
            require_pin: false,
            max_message_bytes: 1024 * 1024,
        }
    }
}

/// The global resource limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LimitsConfig {
    /// The maximum number of decoded video frames buffered in the sink queue.
    pub video_queue_frames: usize,
    /// The maximum total number of bytes buffered in the video sink queue.
    pub video_queue_bytes: usize,
    /// The maximum number of audio packets buffered in the sink queue.
    pub audio_queue_packets: usize,
    /// The maximum total number of bytes buffered in the audio sink queue.
    pub audio_queue_bytes: usize,
    /// The maximum number of pairing attempts before the sender is locked out.
    pub max_pairing_attempts: u32,
    /// The maximum size of an album artwork payload, in bytes.
    pub max_artwork_bytes: usize,
    /// The maximum size of a photo payload, in bytes.
    pub max_photo_bytes: usize,
    /// The maximum size of a DLNA media payload, in bytes.
    pub max_media_bytes: usize,
    /// The maximum number of headers accepted in one HTTP-like request.
    pub max_header_count: usize,
    /// The maximum length of a single header line, in bytes.
    pub max_header_bytes: usize,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            video_queue_frames: 90,
            video_queue_bytes: 64 * 1024 * 1024,
            audio_queue_packets: 200,
            audio_queue_bytes: 4 * 1024 * 1024,
            max_pairing_attempts: 5,
            max_artwork_bytes: 8 * 1024 * 1024,
            max_photo_bytes: 20 * 1024 * 1024,
            max_media_bytes: 512 * 1024 * 1024,
            max_header_count: 64,
            max_header_bytes: 8 * 1024,
        }
    }
}

/// The performance budgets the implementation is measured against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PerformanceConfig {
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

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            first_frame_latency_ms: 100,
            minimum_fps: 25,
            max_memory_mb: 150,
            max_cpu_percent: 30,
            idle_timeout_ms: 30_000,
        }
    }
}

impl PerformanceConfig {
    /// The idle timeout as a duration.
    ///
    /// WHY: the budget is stored in milliseconds because that is what the
    /// configuration file and the dashboards speak. An engine needs a
    /// `Duration`, and converting the number in six places is how the six
    /// places drift apart.
    #[must_use]
    pub fn idle_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.idle_timeout_ms)
    }
}

impl AppConfig {
    /// The configuration of a protocol, with the default applied when absent.
    #[must_use]
    pub fn protocol(&self, protocol: ProtocolKind) -> ProtocolConfig {
        self.protocols
            .get(protocol.as_str())
            .cloned()
            .unwrap_or_default()
    }

    /// Whether a protocol is enabled.
    #[must_use]
    pub fn is_enabled(&self, protocol: ProtocolKind) -> bool {
        self.protocol(protocol).enabled
    }

    /// Validates the configuration.
    ///
    /// # Errors
    /// Returns [`ConfigError::Invalid`] when a value is semantically wrong —
    /// a zero queue budget, an out-of-range latency, and so on.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.limits.video_queue_frames == 0 {
            return Err(ConfigError::Invalid {
                field: "limits.video_queue_frames".into(),
                reason: "must be greater than zero".into(),
            });
        }
        if self.limits.video_queue_bytes == 0 {
            return Err(ConfigError::Invalid {
                field: "limits.video_queue_bytes".into(),
                reason: "must be greater than zero".into(),
            });
        }
        if self.limits.audio_queue_packets == 0 {
            return Err(ConfigError::Invalid {
                field: "limits.audio_queue_packets".into(),
                reason: "must be greater than zero".into(),
            });
        }
        if self.limits.audio_queue_bytes == 0 {
            return Err(ConfigError::Invalid {
                field: "limits.audio_queue_bytes".into(),
                reason: "must be greater than zero".into(),
            });
        }
        if self.limits.max_pairing_attempts == 0 {
            return Err(ConfigError::Invalid {
                field: "limits.max_pairing_attempts".into(),
                reason: "must be greater than zero".into(),
            });
        }
        if self.performance.first_frame_latency_ms == 0 {
            return Err(ConfigError::Invalid {
                field: "performance.first_frame_latency_ms".into(),
                reason: "must be greater than zero".into(),
            });
        }
        if self.performance.minimum_fps == 0 {
            return Err(ConfigError::Invalid {
                field: "performance.minimum_fps".into(),
                reason: "must be greater than zero".into(),
            });
        }
        for (name, protocol) in &self.protocols {
            if let Some(kind) = parse_protocol_name(name) {
                if protocol.max_message_bytes == 0 {
                    return Err(ConfigError::Invalid {
                        field: format!("protocols.{name}.max_message_bytes"),
                        reason: "must be greater than zero".into(),
                    });
                }
                let _ = kind;
            } else {
                return Err(ConfigError::Invalid {
                    field: format!("protocols.{name}"),
                    reason: "unknown protocol name".into(),
                });
            }
        }
        Ok(())
    }

    /// Loads a configuration from a TOML file.
    ///
    /// # Errors
    /// Returns [`ConfigError::NotFound`] if the file does not exist,
    /// [`ConfigError::Parse`] if it is not valid TOML, and any error from
    /// [`AppConfig::validate`].
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => ConfigError::NotFound(path.display().to_string()),
            _ => ConfigError::Io(error),
        })?;
        let config: AppConfig = toml::from_str(&text)?;
        config.validate()?;
        Ok(config)
    }

    /// Serialises the configuration as TOML.
    ///
    /// # Errors
    /// Returns [`ConfigError::Io`] when serialisation fails.
    pub fn to_toml(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self).map_err(|error| ConfigError::Io(std::io::Error::other(error)))
    }
}

/// Parses a protocol name, tolerating the aliases used in configuration.
fn parse_protocol_name(name: &str) -> Option<ProtocolKind> {
    name.parse().ok()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn a_default_configuration_is_valid() {
        assert!(AppConfig::default().validate().is_ok());
    }

    #[test]
    fn an_unknown_protocol_name_is_rejected() {
        let mut config = AppConfig::default();
        config
            .protocols
            .insert("telepathy".to_string(), ProtocolConfig::default());
        let error = config.validate().expect_err("must fail");
        assert!(error.to_string().contains("unknown protocol name"));
    }

    #[test]
    fn a_zero_queue_budget_is_rejected() {
        let mut config = AppConfig::default();
        config.limits.video_queue_frames = 0;
        let error = config.validate().expect_err("must fail");
        assert!(error.to_string().contains("video_queue_frames"));
    }

    #[test]
    fn a_missing_protocol_entry_yields_the_default() {
        let config = AppConfig::default();
        let airplay = config.protocol(ProtocolKind::AirPlay);
        assert!(airplay.enabled);
        assert_eq!(airplay.port, 0);
    }

    #[test]
    fn the_configuration_round_trips_through_toml() {
        let config = AppConfig::default();
        let text = config.to_toml().expect("serialise");
        let parsed: AppConfig = toml::from_str(&text).expect("parse");
        assert_eq!(config, parsed);
    }

    #[test]
    fn an_unknown_key_is_rejected() {
        // WHY: a typo in a configuration file must not be ignored silently.
        // The model is derived with `deny_unknown_fields`, so the parser
        // refuses a key that the model does not know and names it in the
        // error message.
        let text = r#"
            [device]
            name = "Living Room"
            manufakturer = "PhairPlayRust"
        "#;

        let error = toml::from_str::<AppConfig>(text).expect_err("must fail");
        let message = error.to_string();
        assert!(
            message.contains("manufakturer"),
            "the error must name the unknown key: {message}"
        );
    }

    #[test]
    fn a_missing_key_yields_the_default() {
        // WHY: a configuration file that names only what it wants to change
        // must stay valid. A missing key is not an error, a wrong value is.
        let text = r#"
            [device]
            name = "Living Room"
        "#;

        let config: AppConfig = toml::from_str(text).expect("parse");
        assert_eq!(config.device.name, "Living Room");
        assert_eq!(config.device.model, DeviceConfig::default().model);
        assert_eq!(
            config.device.manufacturer,
            DeviceConfig::default().manufacturer
        );
        assert!(config.validate().is_ok());
    }
}
