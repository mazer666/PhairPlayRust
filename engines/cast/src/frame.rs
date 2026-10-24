//! The length-prefixed framing of the CastV2 transport.
//!
//! WHY: on the wire a Cast message is a four-byte big-endian length followed
//! by that many bytes of protobuf. The length is the first thing a peer
//! controls, so it is the first thing that must be bounded: a peer that
//! announces four gigabytes must not make the receiver allocate four
//! gigabytes.
//!
//! WHAT: [`read_frame`] reads one framed message, [`write_frame`] writes
//! one. Both refuse a message that exceeds the configured limit.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_cast::frame::read_frame;
//!
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let mut stream: &[u8] = &[0x00, 0x00, 0x00, 0x02, 0x08, 0x01];
//! let frame = read_frame(&mut stream, 1024).await.expect("frame");
//! assert_eq!(frame, vec![0x08, 0x01]);
//! # });
//! ```

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// The default upper bound for a single CastV2 message.
pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 64 * 1024;

/// A failure while reading or writing a framed message.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    /// The underlying socket failed.
    #[error("cast transport failed: {0}")]
    Io(#[from] std::io::Error),

    /// The peer announced a message that exceeds the limit.
    #[error("cast message of {0} bytes exceeds the limit")]
    TooLarge(usize),

    /// The peer announced an empty message.
    #[error("cast message is empty")]
    Empty,
}

/// Reads one framed CastV2 message.
///
/// The four-byte length prefix is checked against `max_bytes` *before* any
/// buffer is allocated, so a hostile length cannot turn into an allocation.
///
/// # Errors
/// Returns [`FrameError::TooLarge`] when the announced length exceeds the
/// limit, [`FrameError::Empty`] when the length is zero and
/// [`FrameError::Io`] when the socket fails or ends early.
pub async fn read_frame<R>(reader: &mut R, max_bytes: usize) -> Result<Vec<u8>, FrameError>
where
    R: AsyncRead + Unpin,
{
    let mut length_bytes = [0u8; 4];
    reader.read_exact(&mut length_bytes).await?;
    let length = u32::from_be_bytes(length_bytes) as usize;
    if length == 0 {
        return Err(FrameError::Empty);
    }
    if length > max_bytes {
        return Err(FrameError::TooLarge(length));
    }
    let mut payload = vec![0u8; length];
    reader.read_exact(&mut payload).await?;
    Ok(payload)
}

/// Writes one framed CastV2 message.
///
/// # Errors
/// Returns [`FrameError::TooLarge`] when the payload exceeds the limit and
/// [`FrameError::Io`] when the socket fails.
pub async fn write_frame<W>(
    writer: &mut W,
    payload: &[u8],
    max_bytes: usize,
) -> Result<(), FrameError>
where
    W: AsyncWrite + Unpin,
{
    if payload.len() > max_bytes {
        return Err(FrameError::TooLarge(payload.len()));
    }
    let length = u32::try_from(payload.len()).map_err(|_| FrameError::TooLarge(payload.len()))?;
    writer.write_all(&length.to_be_bytes()).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_framed_message_round_trips() {
        let mut buffer: Vec<u8> = Vec::new();
        write_frame(&mut buffer, b"hello", 1024)
            .await
            .expect("write");
        assert_eq!(&buffer[..4], &[0x00, 0x00, 0x00, 0x05]);

        let mut stream: &[u8] = &buffer;
        let frame = read_frame(&mut stream, 1024).await.expect("read");
        assert_eq!(frame, b"hello".to_vec());
    }

    #[tokio::test]
    async fn an_oversized_length_is_rejected_without_allocating() {
        // A peer announces 64 MiB. The reader must refuse before it reserves
        // the memory.
        let bytes = [0xffu8, 0xff, 0xff, 0xff];
        let mut stream: &[u8] = &bytes;
        let error = read_frame(&mut stream, 1024).await.expect_err("must fail");
        match error {
            FrameError::TooLarge(size) => assert_eq!(size, u32::MAX as usize),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn an_empty_frame_is_rejected() {
        let bytes = [0x00u8, 0x00, 0x00, 0x00];
        let mut stream: &[u8] = &bytes;
        assert!(matches!(
            read_frame(&mut stream, 1024).await,
            Err(FrameError::Empty)
        ));
    }

    #[tokio::test]
    async fn a_truncated_frame_is_an_io_error() {
        let bytes = [0x00u8, 0x00, 0x00, 0x08, 0x01];
        let mut stream: &[u8] = &bytes;
        assert!(matches!(
            read_frame(&mut stream, 1024).await,
            Err(FrameError::Io(_))
        ));
    }

    #[tokio::test]
    async fn writing_an_oversized_payload_is_refused() {
        let mut buffer: Vec<u8> = Vec::new();
        let payload = vec![0u8; 32];
        assert!(matches!(
            write_frame(&mut buffer, &payload, 16).await,
            Err(FrameError::TooLarge(32))
        ));
        assert!(buffer.is_empty());
    }
}
