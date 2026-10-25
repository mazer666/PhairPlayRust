//! The WebSocket handshake and the channel of the Smart View receiver.
//!
//! WHAT: the handshake of RFC 6455 and the reading of the frames that
//! follow it.
//!
//! WHY: the sender opens the channel with an HTTP upgrade. The receiver
//! accepts the upgrade, answers with `101 Switching Protocols` and the
//! `Sec-WebSocket-Accept` header, and then holds the connection open. The
//! **content** of the channel is not interpreted: the message format of
//! Smart View is proprietary and not publicly documented, so the frames are
//! read, bounded and discarded — the connection is kept alive and the close
//! is noticed, nothing more is claimed.
//!
//! HOW: every frame is read with a deadline and a length limit. A frame
//! that announces more than the limit is refused instead of being read into
//! memory, and a peer that stops talking loses the channel after the idle
//! timeout.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::api::channel::accept_key;
//!
//! // The example of RFC 6455, section 1.3.
//! assert_eq!(
//!     accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
//!     "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
//! );
//! ```

use base64::Engine as _;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::api::ApiLimits;
use crate::api::message::ChannelMessage;
use crate::error::SmartViewError;

/// The GUID of RFC 6455 that the handshake nonce is combined with.
const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// The opcode of a continuation frame.
pub const OPCODE_CONTINUATION: u8 = 0x0;

/// The opcode of a text frame.
pub const OPCODE_TEXT: u8 = 0x1;

/// The opcode of a binary frame.
pub const OPCODE_BINARY: u8 = 0x2;

/// The opcode of a close frame.
pub const OPCODE_CLOSE: u8 = 0x8;

/// The opcode of a ping frame.
pub const OPCODE_PING: u8 = 0x9;

/// The opcode of a pong frame.
pub const OPCODE_PONG: u8 = 0xA;

/// The maximum payload of a control frame, in bytes.
///
/// WHY: RFC 6455 limits the payload of a ping, a pong and a close frame to
/// 125 bytes. A control frame that announces more is broken.
pub const MAX_CONTROL_PAYLOAD: usize = 125;

/// The value of the `Sec-WebSocket-Accept` header for a client key.
///
/// WHY: RFC 6455 defines the header as the base64 of the SHA-1 of the
/// client key and the fixed GUID. The value proves that the answer belongs
/// to the request; it is **not** a security claim — the channel of Smart
/// View carries no authentication of its own.
#[must_use]
pub fn accept_key(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WEBSOCKET_GUID.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
}

/// One frame of the channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// The opcode of the frame.
    pub opcode: u8,
    /// The length of the payload of the frame.
    pub payload_len: usize,
    /// The payload of a control frame.
    ///
    /// WHY: RFC 6455 limits the payload of a control frame to 125 bytes and
    /// requires the answer to a ping to repeat it.
    pub control_payload: Vec<u8>,
    /// The payload of a data frame.
    ///
    /// WHY the payload is kept: the messages of the channel are JSON, and
    /// the receiver reads them. The payload of a frame is bounded by the
    /// limit of the API, so the buffer is bounded too.
    pub payload: Vec<u8>,
}

/// Reads one frame of the channel.
///
/// # Errors
/// Returns [`SmartViewError`] when the frame is malformed, announces more
/// than the limit allows or does not arrive in time.
pub async fn read_frame(
    reader: &mut BufReader<TcpStream>,
    limits: &ApiLimits,
) -> Result<Frame, SmartViewError> {
    let mut header = [0u8; 2];
    reader
        .read_exact(&mut header)
        .await
        .map_err(|error| SmartViewError::Request(error.to_string()))?;
    let opcode = header[0] & 0x0F;
    let masked = header[1] & 0x80 != 0;
    let mut length = usize::from(header[1] & 0x7F);
    if length == 126 {
        let mut extended = [0u8; 2];
        reader
            .read_exact(&mut extended)
            .await
            .map_err(|error| SmartViewError::Request(error.to_string()))?;
        length = usize::from(u16::from_be_bytes(extended));
    } else if length == 127 {
        let mut extended = [0u8; 8];
        reader
            .read_exact(&mut extended)
            .await
            .map_err(|error| SmartViewError::Request(error.to_string()))?;
        let value = u64::from_be_bytes(extended);
        length = usize::try_from(value).map_err(|_| {
            SmartViewError::LimitExceeded("the frame announces an impossible length".to_string())
        })?;
    }
    if length > limits.max_frame_bytes {
        return Err(SmartViewError::LimitExceeded(format!(
            "the frame announced {} bytes, the limit is {}",
            length, limits.max_frame_bytes
        )));
    }
    if !masked {
        // WHY: RFC 6455 requires every frame of a client to be masked. A
        // frame without a mask is either broken or an attempt to make the
        // receiver read a payload it did not announce.
        return Err(SmartViewError::Protocol(
            "the frame is not masked".to_string(),
        ));
    }
    if is_control(opcode) && length > MAX_CONTROL_PAYLOAD {
        return Err(SmartViewError::LimitExceeded(format!(
            "the control frame announced {length} bytes, the limit is {MAX_CONTROL_PAYLOAD}"
        )));
    }
    let mut mask = [0u8; 4];
    reader
        .read_exact(&mut mask)
        .await
        .map_err(|error| SmartViewError::Request(error.to_string()))?;
    let mut payload = vec![0u8; length];
    if length > 0 {
        reader
            .read_exact(&mut payload)
            .await
            .map_err(|error| SmartViewError::Request(error.to_string()))?;
    }
    unmask(&mut payload, mask);
    let control_payload = if is_control(opcode) {
        payload.clone()
    } else {
        Vec::new()
    };
    Ok(Frame {
        opcode,
        payload_len: length,
        control_payload,
        payload,
    })
}

/// Removes the mask of a frame from its payload.
///
/// WHY: the payload of a frame of a client is masked with a four-byte key
/// that the frame itself carries. The payload of a control frame is read,
/// so it has to be unmasked before it is used.
fn unmask(payload: &mut [u8], mask: [u8; 4]) {
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % 4];
    }
}

/// Whether an opcode belongs to a control frame.
fn is_control(opcode: u8) -> bool {
    matches!(opcode, OPCODE_CLOSE | OPCODE_PING | OPCODE_PONG)
}

/// Holds the channel open until the peer closes it or the idle timeout ends.
///
/// WHY the channel speaks JSON: the messages on the channel of the
/// MultiScreen Framework are JSON objects with an `event` or a `method` and
/// a `data` or a `params`. The client library of Samsung builds the same
/// objects, and the observations of the network show the same. The receiver
/// therefore reads them and answers them.
///
/// WHY the frames are counted and the picture is not transported: the format
/// of the **picture of the screen** of a device is proprietary and is not
/// implemented (`EXC-011`). Everything else on the channel is understood:
/// the events of the sender, the heartbeat, and the keys of the remote
/// control. **No picture is transported by this function, and none is
/// claimed.**
///
/// # Errors
/// Returns [`SmartViewError`] when the channel breaks or a frame violates a
/// limit.
pub async fn hold_channel(
    reader: &mut BufReader<TcpStream>,
    limits: &ApiLimits,
) -> Result<(), SmartViewError> {
    hold_channel_with(reader, limits, &mut |_message| {}).await
}

/// Holds the channel open and hands every message of the sender to the sink.
///
/// WHY the sink: the receiver has to *act* on a message — a key of the
/// remote control belongs to the platform, and the platform of the shell
/// decides what a key does. The channel therefore does not decide anything;
/// it reads the message, understands its shape and hands it on.
///
/// # Errors
/// Returns [`SmartViewError`] when the channel breaks or a frame violates a
/// limit.
pub async fn hold_channel_with(
    reader: &mut BufReader<TcpStream>,
    limits: &ApiLimits,
    sink: &mut (dyn FnMut(ChannelMessage) + Send),
) -> Result<(), SmartViewError> {
    loop {
        let frame = tokio::time::timeout(limits.idle_timeout, read_frame(reader, limits))
            .await
            .map_err(|_| {
                SmartViewError::Request("the peer stopped talking on the channel".to_string())
            })??;
        match frame.opcode {
            OPCODE_CLOSE => {
                let _ = reader.get_mut().write_all(&[0x88, 0x00]).await;
                return Ok(());
            }
            OPCODE_PING => {
                // WHY: RFC 6455 requires the answer to a ping to repeat its
                // payload. The payload of a control frame is at most 125
                // bytes, so the answer stays small.
                let mut pong = vec![0x8A, frame.control_payload.len() as u8];
                pong.extend_from_slice(&frame.control_payload);
                reader
                    .get_mut()
                    .write_all(&pong)
                    .await
                    .map_err(|error| SmartViewError::Request(error.to_string()))?;
            }
            OPCODE_TEXT | OPCODE_BINARY | OPCODE_CONTINUATION | OPCODE_PONG => {
                // WHY the payload is read as text: the messages of the
                // channel are JSON. A frame that carries something else is
                // ignored — a sender that sends an unknown message has not
                // broken the channel.
                if let Ok(text) = std::str::from_utf8(&frame.payload) {
                    if let Some(message) = ChannelMessage::parse(text) {
                        sink(message);
                    }
                }
            }
            other => {
                return Err(SmartViewError::Protocol(format!(
                    "the frame carries the unknown opcode {other}"
                )));
            }
        }
    }
}

/// Writes a text frame to the channel.
///
/// WHY the frame is not masked: RFC 6455 says that a frame of a **server**
/// is never masked. The receiver is the server of the channel, so its
/// frames carry the payload as they are.
///
/// WHY the length is written in the short form first: a message of the
/// channel is small — an event with a token and a key press — and the
/// short form of the length covers everything below 126 bytes. A longer
/// message carries the second byte of the length, and the eight-byte form
/// is the last one.
///
/// # Errors
/// Returns [`SmartViewError`] when the frame cannot be written.
pub async fn write_text_frame(
    writer: &mut TcpStream,
    text: &str,
) -> Result<(), SmartViewError> {
    let payload = text.as_bytes();
    let mut frame = Vec::with_capacity(payload.len() + 10);
    frame.push(0x80 | OPCODE_TEXT);
    if payload.len() < 126 {
        frame.push(payload.len() as u8);
    } else if payload.len() <= u16::MAX as usize {
        frame.push(126);
        frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    }
    frame.extend_from_slice(payload);
    writer
        .write_all(&frame)
        .await
        .map_err(|error| SmartViewError::Request(error.to_string()))
}

/// Writes an event of the channel to the peer.
///
/// # Errors
/// Returns [`SmartViewError`] when the frame cannot be written.
pub async fn write_event(
    writer: &mut TcpStream,
    event: &crate::api::message::ChannelEvent,
) -> Result<(), SmartViewError> {
    write_text_frame(writer, &crate::api::message::ChannelMessage::event(event)).await
}
