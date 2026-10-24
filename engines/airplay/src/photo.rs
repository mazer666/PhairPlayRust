//! The photo endpoint of AirPlay.
//!
//! WHY: a sender shares a photo with `PUT /photo` and a JPEG or PNG body.
//! The body is *untrusted*: any device on the local network can push a
//! megabyte-sized blob of random bytes into the receiver's memory. The
//! validation therefore checks the size against the configured limit and
//! verifies that the payload really is the image type it claims to be —
//! before the bytes are handed to the platform layer.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::photo::{PhotoImageType, PhotoValidation, validate_photo};
//!
//! let jpeg = [0xFFu8, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
//! match validate_photo(&jpeg, Some("image/jpeg"), 1024) {
//!     PhotoValidation::Valid { image_type } => assert_eq!(image_type, PhotoImageType::Jpeg),
//!     PhotoValidation::Invalid { reason } => panic!("{reason}"),
//! }
//! ```

use std::sync::Arc;

/// The path of the photo endpoint.
pub const PHOTO_PATH: &str = "/photo";

/// The image type of a received photo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoImageType {
    /// A JPEG image.
    Jpeg,
    /// A PNG image.
    Png,
}

impl PhotoImageType {
    /// The MIME type of the image.
    #[must_use]
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
        }
    }
}

/// The result of the photo validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhotoValidation {
    /// The payload is a valid image.
    Valid {
        /// The detected image type.
        image_type: PhotoImageType,
    },
    /// The payload was rejected.
    Invalid {
        /// Why the payload was rejected.
        reason: String,
    },
}

/// A photo that passed the validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoPayload {
    /// The image type.
    pub image_type: PhotoImageType,
    /// The image bytes.
    pub data: Arc<Vec<u8>>,
}

/// Validates a photo payload.
///
/// # Parameters
/// * `bytes` — the raw body of the `PUT /photo` request.
/// * `content_type` — the declared content type, if the sender sent one.
/// * `max_bytes` — the maximum size the receiver accepts.
#[must_use]
pub fn validate_photo(
    bytes: &[u8],
    content_type: Option<&str>,
    max_bytes: usize,
) -> PhotoValidation {
    if bytes.is_empty() {
        return PhotoValidation::Invalid {
            reason: "empty photo payload".to_string(),
        };
    }
    if bytes.len() > max_bytes {
        return PhotoValidation::Invalid {
            reason: format!(
                "photo payload of {} bytes exceeds the {max_bytes} byte limit",
                bytes.len()
            ),
        };
    }
    let Some(image_type) = detect_image_type(bytes) else {
        return PhotoValidation::Invalid {
            reason: "unsupported image format".to_string(),
        };
    };
    if let Some(declared) = content_type {
        let declared = declared
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !declared.is_empty()
            && declared != image_type.mime_type()
            && declared != "application/octet-stream"
        {
            return PhotoValidation::Invalid {
                reason: "content type does not match image payload".to_string(),
            };
        }
    }
    PhotoValidation::Valid { image_type }
}

/// Detects the image type of a payload by its magic bytes.
#[must_use]
pub fn detect_image_type(bytes: &[u8]) -> Option<PhotoImageType> {
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Some(PhotoImageType::Jpeg);
    }
    if bytes.len() >= 8 && bytes[..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some(PhotoImageType::Png);
    }
    None
}

#[cfg(test)]
mod tests;
