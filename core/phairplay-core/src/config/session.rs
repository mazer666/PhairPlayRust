//! The configuration of the sessions of the receiver.
//!
//! WHAT: the rule that says how many peers a protocol serves at the same
//! time.
//!
//! WHY: most protocols serve **one** peer at a time — a second connection
//! is refused, not queued. Google Cast is the documented exception: a Cast
//! receiver may serve several senders at the same time. The rule therefore
//! lives in **one** place — the list `multi_peer_protocols` — and every
//! engine asks the configuration instead of deciding for itself.
//!
//! WHY the list is in the configuration and not in the code: a rule that is
//! written twice is a rule that drifts. The list is the owner; the engines
//! are the readers.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::config::AppConfig;
//! use phairplay_core::protocol::ProtocolKind;
//!
//! let mut config = AppConfig::default();
//! config.session.multi_peer_protocols.push(ProtocolKind::Cast.as_str().to_string());
//! assert!(config.serves_several_peers(ProtocolKind::Cast));
//! assert!(!config.serves_several_peers(ProtocolKind::AirPlay));
//! ```

use serde::{Deserialize, Serialize};

use crate::protocol::ProtocolKind;

/// The configuration of the sessions of the receiver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SessionConfig {
    /// The protocols that may serve more than one peer at the same time.
    ///
    /// WHY the list and not a number: the exception is a property of the
    /// protocol, not of the device. A Cast receiver serves several senders
    /// because the protocol says so, not because the device is powerful
    /// enough.
    pub multi_peer_protocols: Vec<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        // WHY the default is the documented exception: Google Cast is the
        // only protocol of the product that serves several senders. The
        // value is the same as the one in `config/app-defaults.toml`, and
        // the test of this module proves that the two agree.
        Self {
            multi_peer_protocols: vec![ProtocolKind::Cast.as_str().to_string()],
        }
    }
}

impl SessionConfig {
    /// Whether the protocol may serve more than one peer at the same time.
    ///
    /// WHY the comparison ignores the case: the name of a protocol comes
    /// from a configuration file, and a file that writes `Cast` and a file
    /// that writes `cast` mean the same protocol.
    #[must_use]
    pub fn serves_several_peers(&self, protocol: ProtocolKind) -> bool {
        self.multi_peer_protocols
            .iter()
            .any(|name| name.eq_ignore_ascii_case(protocol.as_str()))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    use crate::config::AppConfig;
    use crate::protocol::{PeerLimit, ProtocolKind};

    #[test]
    fn the_default_is_the_documented_exception() {
        // WHY: Google Cast is the only protocol of the product that serves
        // several senders at the same time.
        let config = SessionConfig::default();
        assert!(config.serves_several_peers(ProtocolKind::Cast));
        assert!(!config.serves_several_peers(ProtocolKind::AirPlay));
        assert!(!config.serves_several_peers(ProtocolKind::Miracast));
        assert!(!config.serves_several_peers(ProtocolKind::Dlna));
        assert!(!config.serves_several_peers(ProtocolKind::SmartView));
    }

    #[test]
    fn the_name_of_a_protocol_is_compared_without_the_case() {
        // WHY: the name comes from a configuration file, and a file that
        // writes `Cast` and a file that writes `cast` mean the same
        // protocol.
        let config = SessionConfig {
            multi_peer_protocols: vec!["CAST".to_string()],
        };
        assert!(config.serves_several_peers(ProtocolKind::Cast));
    }

    #[test]
    fn the_rule_comes_from_the_configuration_and_not_from_the_engine() {
        // WHY: this is the whole point of the change. The engine asks the
        // configuration, and the configuration answers. A protocol that the
        // list does not name serves one peer — whatever the engine of that
        // protocol believes about itself.
        for protocol in [
            ProtocolKind::AirPlay,
            ProtocolKind::Miracast,
            ProtocolKind::Dlna,
            ProtocolKind::SmartView,
        ] {
            let config = AppConfig::default();
            let limit = PeerLimit::from_config(&config, protocol);
            assert_eq!(
                limit,
                PeerLimit::One,
                "{protocol} may not serve several peers"
            );
            assert_eq!(limit.max_peers(), 1, "{protocol} serves one peer");
        }

        let config = AppConfig::default();
        let limit = PeerLimit::from_config(&config, ProtocolKind::Cast);
        assert!(limit.allows_several_peers());
        assert!(
            limit.max_peers() > 1,
            "a protocol that may serve several peers serves more than one"
        );
    }

    #[test]
    fn the_rule_of_the_configuration_can_be_changed() {
        // WHY: the rule is a value of the configuration, and a value that
        // cannot be changed is a constant in disguise.
        let mut config = AppConfig::default();
        config
            .session
            .multi_peer_protocols
            .push(ProtocolKind::AirPlay.as_str().to_string());
        assert!(config.serves_several_peers(ProtocolKind::AirPlay));
        assert!(!config.serves_several_peers(ProtocolKind::Miracast));
    }

    #[test]
    fn the_default_agrees_with_the_configuration_of_the_repository() {
        // WHY: the default of the code and the value of the file are two
        // statements about the same rule. A test that reads the file of the
        // repository proves that they do not drift apart.
        // WHY the search: the test runs from the directory of the crate, and
        // the file of the repository is two levels above it. The path is
        // searched instead of being written down, because a path that is
        // written down breaks as soon as the crate moves.
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/app-defaults.toml");
        let text =
            std::fs::read_to_string(&path).expect("the file of the defaults must be readable");
        let file: toml::Table = toml::from_str(&text).expect("the file must parse");
        let list = file
            .get("session")
            .and_then(|session| session.get("multi_peer_protocols"))
            .and_then(toml::Value::as_array)
            .expect("the file must name the multi-peer protocols");
        let from_file: Vec<String> = list
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect();
        assert_eq!(
            from_file,
            SessionConfig::default().multi_peer_protocols,
            "the default of the code and the file of the repository disagree"
        );
    }
}
