#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

/// The event that opens the channel carries the token of the session.
#[test]
fn the_connect_event_carries_the_token() {
    let text = ChannelMessage::event(&ChannelEvent::Connect {
        token: "12345678".to_string(),
    });
    assert!(text.contains("\"event\":\"ms.channel.connect\""), "{text}");
    assert!(text.contains("\"token\":\"12345678\""), "{text}");
}

/// The name of the event is the name that the client library expects.
#[test]
fn the_events_carry_the_names_of_the_client_library() {
    assert_eq!(ChannelEvent::Ready.name(), "ms.channel.ready");
    assert_eq!(
        ChannelEvent::Unauthorized {
            reason: "no".to_string()
        }
        .name(),
        "ms.channel.unauthorized"
    );
    assert_eq!(ChannelEvent::Ping.name(), "__ping");
}

/// The message of the sender is read.
#[test]
fn the_event_of_the_sender_is_read() {
    let message = ChannelMessage::parse(r#"{"event":"ms.channel.ready","data":{}}"#)
        .expect("the event must be read");
    assert_eq!(message, ChannelMessage::Event(ChannelEvent::Ready));
}

/// The method of the sender is read, with its parameters.
#[test]
fn the_method_of_the_sender_is_read() {
    let message = ChannelMessage::parse(
        r#"{"method":"ms.remote.control","params":{"Cmd":"Click","DataOfCmd":"KEY_POWER","Option":"false","TypeOfRemote":"SendRemoteKey"}}"#,
    )
    .expect("the method must be read");
    let ChannelMessage::Method { name, params } = &message else {
        panic!("the message is not a method: {message:?}");
    };
    assert_eq!(name, "ms.remote.control");
    assert_eq!(params["Cmd"], "Click");
    assert_eq!(params["DataOfCmd"], "KEY_POWER");
}

/// The key of the remote control is found in the message.
#[test]
fn the_key_of_the_remote_control_is_found() {
    let message = ChannelMessage::parse(
        r#"{"method":"ms.remote.control","params":{"Cmd":"Click","DataOfCmd":"KEY_VOLUP"}}"#,
    )
    .expect("the method must be read");
    assert_eq!(message.remote_control_key(), Some("KEY_VOLUP"));
}

/// A message that carries no key is not a key press.
#[test]
fn a_method_without_a_key_is_no_key_press() {
    let message = ChannelMessage::parse(r#"{"method":"ms.remote.control","params":{}}"#)
        .expect("the method must be read");
    assert_eq!(message.remote_control_key(), None);

    let other = ChannelMessage::parse(
        r#"{"method":"ms.something.else","params":{"DataOfCmd":"KEY_POWER"}}"#,
    )
    .expect("the method must be read");
    assert_eq!(other.remote_control_key(), None);
}

/// A message that cannot be read is ignored — a sender that sends an
/// unknown message has not broken the channel.
#[test]
fn a_message_that_cannot_be_read_is_ignored() {
    assert!(ChannelMessage::parse("not json").is_none());
    assert!(ChannelMessage::parse("{}").is_none());
    assert!(ChannelMessage::parse(r#"{"other":"field"}"#).is_none());
}

/// The event of the sender that is not known does not break the channel.
#[test]
fn an_unknown_event_of_the_sender_is_still_an_event() {
    let message = ChannelMessage::parse(r#"{"event":"ms.channel.something.new"}"#)
        .expect("the event must be read");
    assert!(matches!(message, ChannelMessage::Event(_)));
}
