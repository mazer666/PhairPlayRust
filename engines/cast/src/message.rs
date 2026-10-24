//! The CastV2 message envelope.
//!
//! WHY: every byte that crosses a Cast connection is a `CastMessage`: a
//! protobuf with a source, a destination, a namespace and a payload. The
//! namespace decides who may read the payload; the payload is JSON for the
//! control channels and raw bytes for the media channels.
//!
//! WHAT: [`CastMessage`] is the envelope, [`Namespace`] names the four
//! channels the receiver implements, and [`PayloadType`] distinguishes JSON
//! from binary payloads.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_cast::{CastMessage, Namespace, PayloadType};
//!
//! let message = CastMessage::text(
//!     Namespace::Heartbeat,
//!     "sender-1",
//!     "receiver-0",
//!     r#"{"type":"PING"}"#,
//! );
//! assert_eq!(message.payload_type, Some(PayloadType::String));
//! ```

use crate::proto::{ProtoError, ProtoReader, ProtoWriter, WIRE_LENGTH_DELIMITED, WIRE_VARINT};

/// The transport channel of a Cast connection.
pub const TRANSPORT_CHANNEL: &str = "web-1";
/// The identifier of the virtual channel that carries receiver status.
pub const RECEIVER_CHANNEL: &str = "receiver-0";
/// The identifier of the virtual channel that carries media control.
pub const MEDIA_CHANNEL: &str = "media-0";
/// The identifier of the virtual channel that carries heartbeats.
pub const HEARTBEAT_CHANNEL: &str = "heartbeat-0";

/// The Cast application id of the default media receiver.
///
/// WHY: a sender asks the receiver which applications it offers. The Google
/// "Default Media Receiver" is the application every sender knows how to
/// drive, so it is the default of this engine.
pub const DEFAULT_MEDIA_APP_ID: &str = "CC1AD845";

/// The display name that belongs to [`DEFAULT_MEDIA_APP_ID`].
pub const DEFAULT_MEDIA_APP_NAME: &str = "Default Media Receiver";

/// The protocol version this receiver speaks.
pub const PROTOCOL_VERSION_CASTV2_1_0: u32 = 0;

/// The namespaces the CastV2 receiver implements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    /// The transport channel: `CONNECT`, `CLOSE` and the heartbeat.
    Connection,
    /// The heartbeat channel: `PING` and `PONG`.
    Heartbeat,
    /// The receiver channel: status, volume and application control.
    Receiver,
    /// The media channel: load, play, pause, seek and media status.
    Media,
}

impl Namespace {
    /// The DNS-style name of the namespace.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Connection => "urn:x-cast:com.google.cast.tp.connection",
            Self::Heartbeat => "urn:x-cast:com.google.cast.tp.heartbeat",
            Self::Receiver => "urn:x-cast:com.google.cast.receiver",
            Self::Media => "urn:x-cast:com.google.cast.media",
        }
    }

    /// Resolves a namespace name.
    ///
    /// Returns `None` for a namespace this receiver does not implement; the
    /// caller then closes the channel instead of guessing.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "urn:x-cast:com.google.cast.tp.connection" => Some(Self::Connection),
            "urn:x-cast:com.google.cast.tp.heartbeat" => Some(Self::Heartbeat),
            "urn:x-cast:com.google.cast.receiver" => Some(Self::Receiver),
            "urn:x-cast:com.google.cast.media" => Some(Self::Media),
            _ => None,
        }
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The kind of payload a message carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    /// The payload is UTF-8 JSON.
    String,
    /// The payload is opaque binary data.
    Binary,
}

impl PayloadType {
    /// The wire value of the payload type.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::String => 0,
            Self::Binary => 1,
        }
    }

    /// Resolves a wire value.
    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::String),
            1 => Some(Self::Binary),
            _ => None,
        }
    }
}

/// The authentication type of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    /// The message is not authenticated.
    None,
    /// The message carries a digest.
    Default,
}

impl AuthType {
    /// The wire value of the authentication type.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Default => 1,
        }
    }

    /// Resolves a wire value.
    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Default),
            _ => None,
        }
    }
}

/// A CastV2 message.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CastMessage {
    /// The protocol version of the sender.
    pub protocol_version: Option<u32>,
    /// The channel the message came from.
    pub source_id: Option<String>,
    /// The channel the message is addressed to.
    pub destination_id: Option<String>,
    /// The namespace of the payload.
    pub namespace: Option<String>,
    /// The kind of payload.
    pub payload_type: Option<PayloadType>,
    /// The JSON payload.
    pub payload_utf8: Option<String>,
    /// The binary payload.
    pub payload_binary: Option<Vec<u8>>,
    /// The authentication type of the message.
    pub auth_type: Option<AuthType>,
}

impl CastMessage {
    /// Creates a message with a JSON payload.
    #[must_use]
    pub fn text(
        namespace: Namespace,
        source_id: impl Into<String>,
        destination_id: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            protocol_version: Some(PROTOCOL_VERSION_CASTV2_1_0),
            source_id: Some(source_id.into()),
            destination_id: Some(destination_id.into()),
            namespace: Some(namespace.as_str().to_string()),
            payload_type: Some(PayloadType::String),
            payload_utf8: Some(payload.into()),
            payload_binary: None,
            auth_type: Some(AuthType::None),
        }
    }

    /// Creates a message with a binary payload.
    #[must_use]
    pub fn binary(
        namespace: Namespace,
        source_id: impl Into<String>,
        destination_id: impl Into<String>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            protocol_version: Some(PROTOCOL_VERSION_CASTV2_1_0),
            source_id: Some(source_id.into()),
            destination_id: Some(destination_id.into()),
            namespace: Some(namespace.as_str().to_string()),
            payload_type: Some(PayloadType::Binary),
            payload_utf8: None,
            payload_binary: Some(payload),
            auth_type: Some(AuthType::None),
        }
    }

    /// The namespace of the message, if it names one this receiver knows.
    #[must_use]
    pub fn namespace_kind(&self) -> Option<Namespace> {
        self.namespace.as_deref().and_then(Namespace::parse)
    }

    /// The JSON payload of the message.
    #[must_use]
    pub fn payload(&self) -> Option<&str> {
        self.payload_utf8.as_deref()
    }

    /// Encodes the message into the protobuf wire format.
    ///
    /// # Errors
    /// Returns [`ProtoError::FieldTooLarge`] when a field exceeds the
    /// protobuf field limit.
    pub fn encode(&self) -> Result<Vec<u8>, ProtoError> {
        let mut writer = ProtoWriter::new();
        if let Some(version) = self.protocol_version {
            writer.write_varint_field(1, u64::from(version));
        }
        if let Some(source) = &self.source_id {
            writer.write_string_field(2, source)?;
        }
        if let Some(destination) = &self.destination_id {
            writer.write_string_field(3, destination)?;
        }
        if let Some(namespace) = &self.namespace {
            writer.write_string_field(4, namespace)?;
        }
        if let Some(payload_type) = self.payload_type {
            writer.write_varint_field(5, u64::from(payload_type.as_u32()));
        }
        if let Some(payload) = &self.payload_utf8 {
            writer.write_string_field(6, payload)?;
        }
        if let Some(payload) = &self.payload_binary {
            writer.write_bytes_field(7, payload)?;
        }
        if let Some(auth_type) = self.auth_type {
            writer.write_varint_field(8, u64::from(auth_type.as_u32()));
        }
        Ok(writer.finish())
    }

    /// Decodes a message from the protobuf wire format.
    ///
    /// # Errors
    /// Returns [`ProtoError`] when the message is truncated, when a field
    /// exceeds the protobuf field limit, or when a string field is not valid
    /// UTF-8. Unknown fields are skipped: a newer sender may add fields, and
    /// skipping them is part of the protobuf contract — not a repair.
    pub fn decode(data: &[u8]) -> Result<Self, ProtoError> {
        let mut reader = ProtoReader::new(data);
        let mut message = CastMessage::default();
        while !reader.is_empty() {
            let (field, wire_type) = reader.read_tag()?;
            match (field, wire_type) {
                (1, WIRE_VARINT) => {
                    message.protocol_version = Some(reader.read_varint()? as u32);
                }
                (2, WIRE_LENGTH_DELIMITED) => {
                    message.source_id = Some(reader.read_string()?.to_string());
                }
                (3, WIRE_LENGTH_DELIMITED) => {
                    message.destination_id = Some(reader.read_string()?.to_string());
                }
                (4, WIRE_LENGTH_DELIMITED) => {
                    message.namespace = Some(reader.read_string()?.to_string());
                }
                (5, WIRE_VARINT) => {
                    let raw = reader.read_varint()? as u32;
                    message.payload_type = Some(
                        PayloadType::from_u32(raw)
                            .ok_or(ProtoError::UnsupportedWireType(raw as u8))?,
                    );
                }
                (6, WIRE_LENGTH_DELIMITED) => {
                    message.payload_utf8 = Some(reader.read_string()?.to_string());
                }
                (7, WIRE_LENGTH_DELIMITED) => {
                    message.payload_binary = Some(reader.read_bytes()?.to_vec());
                }
                (8, WIRE_VARINT) => {
                    let raw = reader.read_varint()? as u32;
                    message.auth_type = Some(
                        AuthType::from_u32(raw)
                            .ok_or(ProtoError::UnsupportedWireType(raw as u8))?,
                    );
                }
                (_, wire) => reader.skip_field(wire)?,
            }
        }
        Ok(message)
    }

    /// Swaps source and destination, as a reply requires.
    #[must_use]
    pub fn into_reply(mut self) -> Self {
        std::mem::swap(&mut self.source_id, &mut self.destination_id);
        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn a_message_round_trips_through_the_wire_format() {
        let original = CastMessage::text(
            Namespace::Receiver,
            "sender-1",
            RECEIVER_CHANNEL,
            r#"{"type":"GET_STATUS"}"#,
        );
        let bytes = original.encode().expect("encode");
        let decoded = CastMessage::decode(&bytes).expect("decode");
        assert_eq!(decoded, original);
    }

    #[test]
    fn a_binary_message_round_trips() {
        let original = CastMessage::binary(
            Namespace::Media,
            "sender-1",
            MEDIA_CHANNEL,
            vec![0x00, 0x01, 0x02],
        );
        let bytes = original.encode().expect("encode");
        let decoded = CastMessage::decode(&bytes).expect("decode");
        assert_eq!(decoded.payload_binary, Some(vec![0x00, 0x01, 0x02]));
        assert_eq!(decoded.payload_type, Some(PayloadType::Binary));
    }

    #[test]
    fn a_reply_swaps_source_and_destination() {
        let message = CastMessage::text(Namespace::Heartbeat, "a", "b", "{}");
        let reply = message.into_reply();
        assert_eq!(reply.source_id.as_deref(), Some("b"));
        assert_eq!(reply.destination_id.as_deref(), Some("a"));
    }

    #[test]
    fn every_namespace_resolves_to_its_own_name() {
        for namespace in [
            Namespace::Connection,
            Namespace::Heartbeat,
            Namespace::Receiver,
            Namespace::Media,
        ] {
            assert_eq!(Namespace::parse(namespace.as_str()), Some(namespace));
        }
    }

    #[test]
    fn an_unknown_namespace_is_not_guessed() {
        assert_eq!(
            Namespace::parse("urn:x-cast:com.google.cast.telepathy"),
            None
        );
    }

    #[test]
    fn a_truncated_message_is_rejected() {
        let bytes = vec![0x12, 0x40, b'a'];
        assert_eq!(CastMessage::decode(&bytes), Err(ProtoError::Truncated));
    }

    #[test]
    fn an_unknown_payload_type_is_rejected() {
        // Field 5, wire type 0, value 9: no such payload type exists.
        let bytes = [0x28u8, 0x09];
        assert!(CastMessage::decode(&bytes).is_err());
    }

    #[test]
    fn an_unknown_auth_type_is_rejected() {
        // Field 8, wire type 0, value 9.
        let bytes = [0x40u8, 0x09];
        assert!(CastMessage::decode(&bytes).is_err());
    }

    #[test]
    fn a_message_without_a_namespace_decodes_to_none() {
        let bytes = CastMessage::default().encode().expect("encode");
        let decoded = CastMessage::decode(&bytes).expect("decode");
        assert_eq!(decoded.namespace_kind(), None);
        assert_eq!(decoded.payload(), None);
    }
}
