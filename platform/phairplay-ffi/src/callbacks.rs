//! The callback interfaces of the bridge.
//!
//! WHAT: The three interfaces a host implements to receive the events of the
//! receiver and to decode the media: [`ReceiverEventSink`],
//! [`PlatformVideoSink`] and [`PlatformAudioSink`].
//!
//! WHY: The engines never decode; they hand encoded frames to a sink, and
//! the platform decodes them. A sink that lives in the host — MediaCodec on
//! Android, VideoToolbox on Apple platforms — has to be reachable from the
//! Rust side. A callback interface is the way UniFFI offers for that: the
//! host implements the trait, the bridge adapts it to the sink of the core.
//!
//! The same three interfaces serve the native shells and the dashboard, so
//! that a frame is counted once and in one place.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::callbacks::ReceiverEventSink;
//! use phairplay_ffi::dto::FfiReceiverEvent;
//!
//! // A host implements the interface; the bridge calls it.
//! struct Printer;
//!
//! impl ReceiverEventSink for Printer {
//!     fn on_event(&self, event: FfiReceiverEvent) {
//!         let _ = event;
//!     }
//! }
//! ```

use crate::dto::{
    FfiAudioFormat, FfiEncodedAudioFrame, FfiEncodedVideoFrame, FfiReceiverEvent, FfiVideoFormat,
};
use crate::error::FfiError;

/// Receives the live events of the receiver.
///
/// WHY: The dashboard shows a live list of sessions, the phone shows a
/// notification when a sender appears. Both need the same events, so the
/// bridge hands them out through one interface.
#[uniffi::export(callback_interface)]
pub trait ReceiverEventSink: Send + Sync {
    /// Called for every event of the receiver.
    ///
    /// WHY: The call happens on a thread of the bridge, not on the thread
    /// of the host. A host that blocks here blocks the receiver.
    fn on_event(&self, event: FfiReceiverEvent);
}

/// Decodes the video frames of the receiver.
#[uniffi::export(callback_interface)]
pub trait PlatformVideoSink: Send + Sync {
    /// Delivers one encoded video frame to the decoder of the platform.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the decoder refuses the frame. The frame
    /// is counted as dropped, and the engine is told that the sink is
    /// closed.
    fn submit_frame(&self, frame: FfiEncodedVideoFrame) -> Result<(), FfiError>;

    /// Announces the format the following frames use.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the decoder does not support the format.
    fn set_video_format(&self, format: FfiVideoFormat) -> Result<(), FfiError>;

    /// Clears the buffered state of the decoder. Called when a session ends.
    fn reset_video(&self);
}

/// Decodes the audio packets of the receiver.
#[uniffi::export(callback_interface)]
pub trait PlatformAudioSink: Send + Sync {
    /// Delivers one encoded audio packet to the decoder of the platform.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the decoder refuses the packet.
    fn submit_packet(&self, packet: FfiEncodedAudioFrame) -> Result<(), FfiError>;

    /// Announces the format the following packets use.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the decoder does not support the format.
    fn set_audio_format(&self, format: FfiAudioFormat) -> Result<(), FfiError>;

    /// Sets the output volume, where `1.0` is the nominal volume.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the output device refuses the volume.
    fn set_volume(&self, volume: f32) -> Result<(), FfiError>;

    /// Clears the buffered state of the decoder. Called when a session ends.
    fn reset_audio(&self);
}
