//! The FairPlay key exchange.
//!
//! WHY: an AirPlay sender that plays protected content runs the FairPlay
//! handshake (`POST /fp-setup`) before it hands over the stream key. The
//! handshake has two phases and two versions:
//!
//! * **v3** (`FPLY 03 …`) — screen mirroring and Safari video. The first
//!   phase has four mode-specific answers, selected by byte 14 of the
//!   request.
//! * **v2** (`FPLY 02 …`) — the RAOP audio path that Apple Music and iTunes
//!   use. The first phase has a single answer whose mode byte (offset 13) is
//!   patched with the mode of the request.
//!
//! Both versions share the second phase and the key decryption.
//!
//! ## The DRM boundary
//!
//! The first and the second phase are pure protocol: the answers are fixed
//! blobs and the second phase echoes the sender's key message. The *decryption*
//! of the wrapped stream key is not: it needs Apple's proprietary FairPlay
//! cipher and its lookup tables, which are neither published nor
//! re-implementable from the protocol description. This engine therefore
//! performs the handshake and reports the boundary explicitly instead of
//! pretending that the key was recovered. A stream whose key cannot be
//! unwrapped stays encrypted — that is an honest product limit, not a bug.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::fairplay::FairPlaySession;
//!
//! let mut session = FairPlaySession::new();
//! let mut request = [0u8; 16];
//! request[4] = 0x03; // version 3
//! request[14] = 0; // mode
//! let answer = session.setup(&request).expect("phase 1");
//! assert_eq!(answer.len(), 142);
//! ```

use crate::error::AirPlayError;

/// The size of the first phase request.
pub const SETUP_REQUEST_BYTES: usize = 16;

/// The size of the second phase request.
pub const HANDSHAKE_REQUEST_BYTES: usize = 164;

/// The size of the first phase answer.
pub const SETUP_REPLY_BYTES: usize = 142;

/// The size of the second phase answer.
pub const HANDSHAKE_REPLY_BYTES: usize = 32;

/// The version of the mirroring handshake.
pub const VERSION_V3: u8 = 0x03;

/// The version of the audio handshake.
pub const VERSION_V2: u8 = 0x02;

/// The offset of the mode indicator in the second version answer.
const V2_MODE_OFFSET: usize = 13;

/// The offset of the version byte in a request.
const VERSION_OFFSET: usize = 4;

/// The offset of the mode byte in a request.
const MODE_OFFSET: usize = 14;

/// The offset of the echoed key material in the second phase request.
const HANDSHAKE_ECHO_OFFSET: usize = 144;

/// The FairPlay state of one control connection.
#[derive(Debug, Default)]
pub struct FairPlaySession {
    key_message: Option<Vec<u8>>,
    version: u8,
}

impl FairPlaySession {
    /// Creates an empty session.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The version negotiated in the first phase.
    #[must_use]
    pub fn negotiated_version(&self) -> u8 {
        self.version
    }

    /// Whether the second phase has completed.
    #[must_use]
    pub fn has_key_message(&self) -> bool {
        self.key_message.is_some()
    }

    /// The first phase of the handshake: a 16-byte request, a 142-byte answer.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Parse`] when the request has the wrong size or
    /// carries an unsupported version or mode.
    pub fn setup(&mut self, request: &[u8]) -> Result<Vec<u8>, AirPlayError> {
        if request.len() != SETUP_REQUEST_BYTES {
            return Err(AirPlayError::parse(format!(
                "fp-setup phase 1 expects {SETUP_REQUEST_BYTES} bytes, got {}",
                request.len()
            )));
        }
        let version = request[VERSION_OFFSET];
        let mode = request[MODE_OFFSET];
        if mode > 3 {
            return Err(AirPlayError::parse(format!(
                "fp-setup phase 1 carries the invalid mode {mode}"
            )));
        }
        self.version = version;
        match version {
            VERSION_V3 => Ok(v3_reply(mode)),
            VERSION_V2 => {
                let mut reply = v2_reply();
                reply[V2_MODE_OFFSET] = mode;
                Ok(reply)
            }
            other => Err(AirPlayError::parse(format!(
                "unsupported FairPlay version {other}"
            ))),
        }
    }

    /// The second phase of the handshake: a 164-byte request, a 32-byte answer.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Parse`] when the request has the wrong size or
    /// carries an unsupported version.
    pub fn handshake(&mut self, request: &[u8]) -> Result<Vec<u8>, AirPlayError> {
        if request.len() != HANDSHAKE_REQUEST_BYTES {
            return Err(AirPlayError::parse(format!(
                "fp-setup phase 2 expects {HANDSHAKE_REQUEST_BYTES} bytes, got {}",
                request.len()
            )));
        }
        let version = request[VERSION_OFFSET];
        if version != VERSION_V2 && version != VERSION_V3 {
            return Err(AirPlayError::parse(format!(
                "unsupported FairPlay version {version}"
            )));
        }
        self.key_message = Some(request.to_vec());
        self.version = version;

        let mut reply = fp_header();
        reply[VERSION_OFFSET] = version;
        reply.extend_from_slice(&request[HANDSHAKE_ECHO_OFFSET..]);
        Ok(reply)
    }

    /// Unwraps the FairPlay-wrapped stream key.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Unsupported`] — the decryption needs Apple's
    /// proprietary cipher, which this engine deliberately does not ship. The
    /// caller keeps the stream encrypted and tells the user why.
    pub fn decrypt(&self, wrapped_key: &[u8]) -> Result<Vec<u8>, AirPlayError> {
        if self.key_message.is_none() {
            return Err(AirPlayError::invalid_state(
                "the fp-setup handshake must complete before the key can be unwrapped",
            ));
        }
        if wrapped_key.len() != 72 {
            return Err(AirPlayError::parse(format!(
                "a FairPlay wrapped key is 72 bytes, got {}",
                wrapped_key.len()
            )));
        }
        Err(AirPlayError::unsupported(
            "FairPlay stream-key decryption needs Apple's proprietary cipher; \
             the content stays encrypted",
        ))
    }
}

/// The first phase answer of the mirroring handshake for one mode.
fn v3_reply(mode: u8) -> Vec<u8> {
    match mode {
        0 => v3_reply_0(),
        1 => v3_reply_1(),
        2 => v3_reply_2(),
        3 => v3_reply_3(),
        _ => v3_reply_0(),
    }
}

/// Decodes a hexadecimal string.
fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(text.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = (pair[0] as char).to_digit(16)?;
        let low = (pair[1] as char).to_digit(16)?;
        out.push(((high << 4) | low) as u8);
    }
    Some(out)
}

fn v3_reply_0() -> Vec<u8> {
    const HEX: &str = "\
    46504c59030102000000008202000f9f3f9e0a2521dbdf312ab2bfb29e8d232b\
    6376a8c818701d22ae93d82737feaf9db4fdf41c2dba9d1f49caaabf6591ac1f\
    7bc6f7e0663d21afe01565953eab81f418ceed095adb7c3d0e254909a79831d4\
    9c3982973434facb42c63a1cd911a6fe941a8a6d4a743b46c3a7649e44c78955\
    e49d8155009549c4e2f7a3f6d5ba";
    decode_hex(HEX).unwrap_or_default()
}

fn v3_reply_1() -> Vec<u8> {
    const HEX: &str = "\
    46504c5903010200000000820201cf32a25714b2524f8aa0ad7af164e37bcf44\
    24e200047efc0ad67afcd95ded1c2730bb591b962ed63a9c4ded88ba8fc78de6\
    4d91ccfd5c7b56da88e31f5cceafc7431995a01665a54e1939d25b94db64b9e4\
    5d8d063e1e6af07e9656162b0efa404275ea5a44d9591c7256b9fbe6513898b8\
    0227721988571650942ad946688a";
    decode_hex(HEX).unwrap_or_default()
}

fn v3_reply_2() -> Vec<u8> {
    const HEX: &str = "\
    46504c5903010200000000820202c169a352eeed35b18cdd9c58d64f16c1519a\
    89eb5317bd0d4336cd68f638ff9d016a5b52b7fa9216b2b65482c78444118121\
    a2c7fed83db7119e9182aad7d18c7063e2a457555910af9e0efc76347d164043\
    807f581ee4fbe42ca9dedc1b5eb2a3aa3d2ecd59e7eee70b3629f22afd161d87\
    7353ddb99adc8e07006e56f850ce";
    decode_hex(HEX).unwrap_or_default()
}

fn v3_reply_3() -> Vec<u8> {
    const HEX: &str = "\
    46504c59030102000000008202039001e1727e0f57f9f5880db104a6257a23f5\
    cfff1abbe1e93045251afb97eb9fc0011ebe0f3a81df5b691d76acb2f7a5c708\
    e3d328f56bb39dbde5f29c8a17f481487e3ae863c678325422e6f78e166d18aa\
    7fd636258bce28726f661f738893ce44311e4be6c0535193e5ef72e868623372\
    9c227d820c999445d89246c8c359";
    decode_hex(HEX).unwrap_or_default()
}

fn v2_reply() -> Vec<u8> {
    const HEX: &str = "\
    46504c59020102000000008202022f7b69e6b27ebbf0685f98547f37cecf8706\
    996e7e6b0fb2fa712053e39483da22c783a072404ddd41aa3d4c6e302255aaa2\
    da1eb477838c79d56517c3fa0154339ee3829f30f0a48f76df77117e569ef395\
    e8e213b31eb670ec5a8af26afcbc8931e67ee8b9c5f2c71d78f3ef8d61f73bcc\
    17c34023524a8b9cb1750566e6b3";
    decode_hex(HEX).unwrap_or_default()
}

fn fp_header() -> Vec<u8> {
    const HEX: &str = "\
    46504c590301040000000014";
    decode_hex(HEX).unwrap_or_default()
}

#[cfg(test)]
mod tests;
