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
pub mod profiles;
pub mod service_types;
pub mod timeouts;

pub use engine_limits::EngineLimits;
pub use profiles::DeviceProfile;
pub use session::SessionConfig;

mod session;

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
    /// The rule that says how many peers a protocol serves at the same time.
    pub session: SessionConfig,
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

    /// Whether a protocol may serve more than one peer at the same time.
    ///
    /// WHY the method on the configuration and not in the engines: the rule
    /// has **one** owner — the list `session.multi_peer_protocols` — and every
    /// engine asks it. A rule that is written twice is a rule that drifts.
    ///
    /// WHY the comparison ignores the case: the name of a protocol comes
    /// from a configuration file.
    #[must_use]
    pub fn serves_several_peers(&self, protocol: ProtocolKind) -> bool {
        self.session.serves_several_peers(protocol)
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
        let text = read_file(path)?;
        let config: AppConfig = toml::from_str(&text)?;
        config.validate()?;
        Ok(config)
    }

    /// Loads a configuration from a TOML file and lays it over a base.
    ///
    /// WHY: a device profile is a *base*, and a configuration file is the
    /// explicit wish of the user. The values of the file therefore win over
    /// the values of the base, and the base fills in what the file does
    /// not mention. The merge happens on the tables and not on the typed
    /// model, because only the table knows which value the user actually
    /// wrote — a model that was deserialised with defaults cannot tell a
    /// default from an explicit value.
    ///
    /// # Errors
    /// Returns [`ConfigError::NotFound`] if the file does not exist,
    /// [`ConfigError::Parse`] if it is not valid TOML or does not fit the
    /// model, and any error from [`AppConfig::validate`].
    pub fn from_path_over(path: impl AsRef<Path>, base: &toml::Table) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let text = read_file(path)?;
        let overlay: toml::Table = toml::from_str(&text)?;
        let mut merged = base.clone();
        Self::merge_tables(&mut merged, overlay);
        let config: AppConfig = merged.try_into()?;
        config.validate()?;
        Ok(config)
    }

    /// Builds a configuration out of a table.
    ///
    /// # Errors
    /// Returns [`ConfigError::Parse`] when the table does not fit the model
    /// and any error from [`AppConfig::validate`].
    pub fn from_table(table: toml::Table) -> Result<Self, ConfigError> {
        let config: AppConfig = table.try_into()?;
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

    /// The configuration as a table.
    ///
    /// WHY: a table is what the merge of a base and an overlay works on.
    /// The configuration is therefore able to become the base of a file
    /// without a detour through the text of the file.
    ///
    /// # Errors
    /// Returns [`ConfigError::Io`] when serialisation fails.
    pub fn to_table(&self) -> Result<toml::Table, ConfigError> {
        let text = self.to_toml()?;
        toml::from_str(&text).map_err(|error| ConfigError::Io(std::io::Error::other(error)))
    }

    /// Merges two tables: the values of the overlay win.
    ///
    /// WHY: the merge is deep for the tables and shallow for the values. A
    /// section of the overlay replaces the section of the base *key by
    /// key*, so that a file that changes one limit keeps the other limits
    /// of the base. A value that is not a table replaces the value of the
    /// base as a whole.
    pub fn merge_tables(base: &mut toml::Table, overlay: toml::Table) {
        for (key, value) in overlay {
            match (base.get_mut(&key), value) {
                (Some(toml::Value::Table(base_table)), toml::Value::Table(overlay_table)) => {
                    Self::merge_tables(base_table, overlay_table);
                }
                (_, value) => {
                    base.insert(key, value);
                }
            }
        }
    }
}

/// Reads a configuration file.
///
/// WHY: the failure of a file that does not exist is not the same failure
/// as the failure of a file that cannot be read. The first is a typo in
/// the command line, the second is a broken installation — and the user
/// has to be able to tell them apart.
fn read_file(path: &Path) -> Result<String, ConfigError> {
    std::fs::read_to_string(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => ConfigError::NotFound(path.display().to_string()),
        _ => ConfigError::Io(error),
    })
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

    /// Writes a fixture into the temporary directory and returns its path.
    ///
    /// WHY: the loader needs a file, and the repository must not be filled
    /// with the fixtures of a test. The name carries the process id, so
    /// two fixtures of the same test cannot collide.
    fn fixture(name: &str, content: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("phairplay-config-{}-{name}", std::process::id()));
        std::fs::write(&path, content).expect("the fixture must be writable");
        path
    }

    /// Removes a fixture again.
    fn remove(path: &std::path::Path) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_configuration_is_loaded_from_a_file() {
        let path = fixture(
            "valid.toml",
            r#"
                [device]
                name = "Living Room"

                [performance]
                max_memory_mb = 64
            "#,
        );
        let config = AppConfig::from_path(&path).expect("the file must be loaded");
        remove(&path);

        assert_eq!(config.device.name, "Living Room");
        assert_eq!(config.performance.max_memory_mb, 64);
        // WHY: a key that the file does not mention keeps the default of
        // the model — the file changes what it names and nothing else.
        assert_eq!(
            config.device.manufacturer,
            DeviceConfig::default().manufacturer
        );
    }

    #[test]
    fn a_missing_file_names_the_path() {
        let mut path = std::env::temp_dir();
        path.push(format!("phairplay-absent-{}.toml", std::process::id()));
        let error = AppConfig::from_path(&path).expect_err("the file must be missing");
        let message = error.to_string();
        assert!(message.contains("not found"), "{message}");
        assert!(message.contains("phairplay-absent"), "{message}");
    }

    #[test]
    fn a_broken_file_is_a_parse_error() {
        let path = fixture("broken.toml", "[device\nname = \"Living Room\"");
        let error = AppConfig::from_path(&path).expect_err("the file must be refused");
        remove(&path);
        assert!(matches!(error, ConfigError::Parse(_)), "{error}");
    }

    #[test]
    fn a_file_with_an_invalid_value_is_refused() {
        let path = fixture("invalid.toml", "[limits]\nvideo_queue_frames = 0");
        let error = AppConfig::from_path(&path).expect_err("the value must be refused");
        remove(&path);
        assert!(error.to_string().contains("video_queue_frames"), "{error}");
    }

    #[test]
    fn a_file_lays_over_a_base() {
        // WHY: the profile is the base and the file is the explicit wish of
        // the user. The file wins — and only for the keys it names.
        let mut base = AppConfig::default();
        base.device.name = "PhairPlay Pi".to_string();
        base.device.model = "Raspberry Pi Receiver".to_string();
        base.performance.max_cpu_percent = 25;
        let base = base.to_table().expect("a table");

        let path = fixture(
            "overlay.toml",
            r#"
                [device]
                name = "Living Room"

                [performance]
                max_memory_mb = 64
            "#,
        );
        let config = AppConfig::from_path_over(&path, &base).expect("the file must be loaded");
        remove(&path);

        assert_eq!(config.device.name, "Living Room");
        assert_eq!(config.performance.max_memory_mb, 64);
        // WHY: the values of the base that the file does not mention stay.
        assert_eq!(config.device.model, "Raspberry Pi Receiver");
        assert_eq!(config.performance.max_cpu_percent, 25);
    }

    #[test]
    fn the_merge_is_deep() {
        let mut base = toml::Table::new();
        base.insert(
            "device".to_string(),
            toml::Value::Table({
                let mut device = toml::Table::new();
                device.insert("name".to_string(), toml::Value::String("Pi".to_string()));
                device.insert(
                    "model".to_string(),
                    toml::Value::String("Board".to_string()),
                );
                device
            }),
        );
        let mut overlay = toml::Table::new();
        overlay.insert(
            "device".to_string(),
            toml::Value::Table({
                let mut device = toml::Table::new();
                device.insert("name".to_string(), toml::Value::String("Attic".to_string()));
                device
            }),
        );

        AppConfig::merge_tables(&mut base, overlay);

        let device = base
            .get("device")
            .and_then(toml::Value::as_table)
            .expect("a table");
        assert_eq!(
            device.get("name").and_then(toml::Value::as_str),
            Some("Attic")
        );
        assert_eq!(
            device.get("model").and_then(toml::Value::as_str),
            Some("Board")
        );
    }

    #[test]
    fn a_configuration_round_trips_through_a_table() {
        let config = AppConfig::default();
        let table = config.to_table().expect("a table");
        let parsed = AppConfig::from_table(table).expect("a configuration");
        assert_eq!(config, parsed);
    }
}
