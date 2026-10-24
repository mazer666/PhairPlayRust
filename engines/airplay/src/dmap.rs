//! The DMAP/DAAP now-playing metadata.
//!
//! WHY: an audio-only AirPlay session carries its track information in a
//! binary `SET_PARAMETER` body: a flat stream of four-character tags, each
//! followed by a big-endian length and a payload. Container tags such as
//! `mlit` nest other tags. The receiver needs three of them — the track name
//! (`minm`), the artist (`asar`) and the album (`asal`) — to show a
//! now-playing screen.
//!
//! The body is untrusted input, so the walker is bounded in three ways: the
//! nesting depth is limited, every length is checked against the buffer end
//! before it is used, and a malformed length ends the scan of that level
//! instead of reading past the buffer.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::dmap::parse_now_playing;
//!
//! // "minm" + length 5 + "Title"
//! let body = [b'm', b'i', b'n', b'm', 0, 0, 0, 5, b'T', b'i', b't', b'l', b'e'];
//! let metadata = parse_now_playing(&body).expect("valid dmap");
//! assert_eq!(metadata.title.as_deref(), Some("Title"));
//! ```

use crate::error::AirPlayError;

/// The maximum nesting depth of a DMAP document.
pub const MAX_DEPTH: u32 = 8;

/// The tags that carry the now-playing information.
const TRACK_TAG: &[u8; 4] = b"minm";
const ARTIST_TAG: &[u8; 4] = b"asar";
const ALBUM_TAG: &[u8; 4] = b"asal";

/// The tags that hold other tags.
const CONTAINER_TAGS: [&[u8; 4]; 4] = [b"mlit", b"mlcl", b"mshl", b"adbs"];

/// The metadata of the track that is playing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NowPlaying {
    /// The track title.
    pub title: Option<String>,
    /// The track artist.
    pub artist: Option<String>,
    /// The album title.
    pub album: Option<String>,
}

impl NowPlaying {
    /// Whether the document carried any of the three fields.
    #[must_use]
    pub fn has_any(&self) -> bool {
        self.title.is_some() || self.artist.is_some() || self.album.is_some()
    }

    /// Whether the sender supplied at least a title.
    #[must_use]
    pub fn has_metadata(&self) -> bool {
        self.title
            .as_ref()
            .is_some_and(|title| !title.trim().is_empty())
    }
}

/// Parses a DMAP now-playing body.
///
/// # Errors
/// Returns [`AirPlayError::Parse`] when the body is not a DMAP document at
/// all — the first tag is not four printable characters, or its length
/// overruns the buffer.
pub fn parse_now_playing(body: &[u8]) -> Result<NowPlaying, AirPlayError> {
    if body.len() < 8 {
        return Err(AirPlayError::parse("DMAP body is shorter than one tag"));
    }
    let mut metadata = NowPlaying::default();
    let malformed = walk(body, 0, body.len(), &mut metadata, 0);
    if malformed && !metadata.has_any() {
        return Err(AirPlayError::parse("malformed DMAP body"));
    }
    Ok(metadata)
}

/// Walks one level of a DMAP document.
///
/// Returns `true` when the level contains a malformed tag — a length that
/// overruns the buffer or a tag that is not printable ASCII.
fn walk(body: &[u8], start: usize, end: usize, metadata: &mut NowPlaying, depth: u32) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    let mut position = start;
    while position + 8 <= end {
        let tag = &body[position..position + 4];
        if !tag.iter().all(|byte| byte.is_ascii_graphic() || *byte == b' ') {
            return true;
        }
        let length = read_u32(&body[position + 4..position + 8]) as usize;
        let payload_start = position + 8;
        if payload_start + length > end {
            return true;
        }
        let payload = &body[payload_start..payload_start + length];
        if tag == TRACK_TAG {
            if metadata.title.is_none() {
                metadata.title = decode_text(payload);
            }
        } else if tag == ARTIST_TAG {
            if metadata.artist.is_none() {
                metadata.artist = decode_text(payload);
            }
        } else if tag == ALBUM_TAG {
            if metadata.album.is_none() {
                metadata.album = decode_text(payload);
            }
        } else if CONTAINER_TAGS.iter().any(|container| *container == tag) {
            walk(body, payload_start, payload_start + length, metadata, depth + 1);
        }
        position = payload_start + length;
    }
    false
}

/// Reads a big-endian `u32`.
fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Decodes a DMAP text payload, mapping an empty value to `None`.
fn decode_text(payload: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(payload);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests;
