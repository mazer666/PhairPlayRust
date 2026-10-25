//! The messages on a channel of the MultiScreen Framework.
//!
//! WHAT: the codec of the JSON messages that the sender and the receiver
//! exchange on the WebSocket channel — the events that the receiver sends,
//! and the methods that it understands.
//!
//! WHY JSON and not a binary format: the wire format of the channel is
//! **documented by the behaviour that is observed on the network**, and the
//! observations show JSON objects with three kinds of fields — `event` for
//! something that happened, `method` for something that is asked for, and
//! `data` or `params` for the content. The client library of Samsung builds
//! the same objects. The receiver therefore speaks JSON, and it speaks the
//! names of the fields that a client of Samsung expects.
//!
//! WHY the events are named the way they are: a client does not read the
//! events of the receiver — it reacts to them. The name of the event is
//! therefore part of the contract, and the names are the ones that were
//! observed:
//!
//! - `ms.channel.connect` — the receiver tells the sender that the channel
//!   is open, and hands it the token of the session and the name of the
//!   channel;
//! - `ms.channel.ready` — the sender is ready to talk;
//! - `ms.channel.unauthorized` — the sender is not allowed to use the
//!   channel;
//! - `__ping` — the heartbeat of the client library.
//!
//! WHY the method `ms.remote.control` is understood: it is the method that
//! carries the keys of the remote control of a television. A receiver that
//! answers it can be driven with the remote of a phone — and that is a
//! capability that does not need the picture of the screen.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::api::message::{ChannelEvent, ChannelMessage};
//!
//! let event = ChannelEvent::Connect { token: "12345678".to_string() };
//! let text = ChannelMessage::event(&event);
//! assert!(text.contains("\"event\":\"ms.channel.connect\""));
//! ```

use serde_json::{Value, json};

/// The name of the event that opens the channel.
pub const EVENT_CONNECT: &str = "ms.channel.connect";

/// The name of the event that says that the sender is ready.
pub const EVENT_READY: &str = "ms.channel.ready";

/// The name of the event that refuses the sender.
pub const EVENT_UNAUTHORIZED: &str = "ms.channel.unauthorized";

/// The name of the heartbeat.
pub const EVENT_PING: &str = "__ping";

/// The name of the method that carries the keys of the remote control.
pub const METHOD_REMOTE_CONTROL: &str = "ms.remote.control";

/// The events that the receiver sends on the channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelEvent {
    /// The channel is open. The token identifies the session.
    Connect {
        /// The token of the session.
        token: String,
    },
    /// The sender is ready to talk.
    Ready,
    /// The sender is not allowed to use the channel.
    Unauthorized {
        /// The reason of the refusal.
        reason: String,
    },
    /// The heartbeat of the client library.
    Ping,
}

impl ChannelEvent {
    /// The name of the event, as the client library expects it.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Connect { .. } => EVENT_CONNECT,
            Self::Ready => EVENT_READY,
            Self::Unauthorized { .. } => EVENT_UNAUTHORIZED,
            Self::Ping => EVENT_PING,
        }
    }

    /// The event as the JSON object that goes over the wire.
    #[must_use]
    pub fn to_json(&self) -> Value {
        match self {
            Self::Connect { token } => json!({
                "event": EVENT_CONNECT,
                "data": {
                    "id": token,
                    "token": token,
                },
            }),
            Self::Ready => json!({ "event": EVENT_READY, "data": {} }),
            Self::Unauthorized { reason } => json!({
                "event": EVENT_UNAUTHORIZED,
                "data": { "reason": reason },
            }),
            Self::Ping => json!({ "event": EVENT_PING, "data": {} }),
        }
    }
}

/// A message of the channel, in either direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMessage {
    /// An event that the receiver sends.
    Event(ChannelEvent),
    /// A method that the sender asks for.
    Method {
        /// The name of the method.
        name: String,
        /// The parameters of the method.
        params: Value,
    },
}

impl ChannelMessage {
    /// The message of an event, as the text that goes over the wire.
    #[must_use]
    pub fn event(event: &ChannelEvent) -> String {
        event.to_json().to_string()
    }

    /// The message of a method, as the text that goes over the wire.
    #[must_use]
    pub fn method(name: &str, params: Value) -> String {
        json!({ "method": name, "params": params }).to_string()
    }

    /// Reads a message of the sender.
    ///
    /// WHY the tolerance: a sender may send a message that the receiver does
    /// not know — a newer version of the client library sends events that an
    /// older receiver has never heard of. A message that cannot be read is
    /// **ignored**, not an error: a sender that sends one unknown message has
    /// not broken the channel.
    ///
    /// EXAMPLE: the message `{"method":"ms.remote.control","params":{…}}` is
    /// read as a method with the name `ms.remote.control`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let value: Value = serde_json::from_str(text).ok()?;
        if let Some(name) = value.get("event").and_then(Value::as_str) {
            return Some(Self::Event(match name {
                EVENT_READY => ChannelEvent::Ready,
                EVENT_UNAUTHORIZED => ChannelEvent::Unauthorized {
                    reason: value
                        .get("data")
                        .and_then(|data| data.get("reason"))
                        .and_then(Value::as_str)
                        .unwrap_or("the channel is not allowed")
                        .to_string(),
                },
                EVENT_PING => ChannelEvent::Ping,
                _ => ChannelEvent::Connect {
                    token: value
                        .get("data")
                        .and_then(|data| data.get("token"))
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                },
            }));
        }
        let name = value.get("method").and_then(Value::as_str)?;
        let params = value
            .get("params")
            .cloned()
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
        Some(Self::Method {
            name: name.to_string(),
            params,
        })
    }

    /// The key that a method of the remote control asks for.
    ///
    /// WHY the field: the observation of the channel shows the key in the
    /// parameter `DataOfCmd` — the command that is sent, for example
    /// `KEY_POWER`. The name of the field is part of the contract, and it is
    /// the only place where the key stands.
    #[must_use]
    pub fn remote_control_key(&self) -> Option<&str> {
        let Self::Method { name, params } = self else {
            return None;
        };
        if name != METHOD_REMOTE_CONTROL {
            return None;
        }
        params.get("DataOfCmd").and_then(Value::as_str)
    }
}

#[cfg(test)]
mod tests;
