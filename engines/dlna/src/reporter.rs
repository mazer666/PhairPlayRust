//! The session reporter of the DLNA engine.
//!
//! WHY: the build contract asks every engine to report a `Started` event
//! when a peer is accepted and an `Ended` event when the session
//! terminates. For the renderer the session begins when a control point
//! claims the transport and ends when it releases it.
//!
//! HOW: the reporter watches the state of the renderer and turns the
//! difference into events. It is an event sink like every other, so the
//! dispatcher does not have to know about it.

use std::sync::{Arc, Mutex};

use phairplay_core::protocol::ProtocolKind;
use phairplay_core::session::{SessionEndReason, SessionEvent, SessionId, SessionSnapshot};

use crate::soap::EventSink;
use crate::state::RendererState;

/// Reports the session lifecycle of the renderer.
///
/// WHY: the build contract asks every engine to report a `Started` event
/// when a peer is accepted and an `Ended` event when the session
/// terminates. For the renderer the session begins when a control point
/// claims the transport and ends when it releases it.
#[derive(Debug)]
pub struct SessionReporter {
    events: Mutex<Option<tokio::sync::mpsc::Sender<SessionEvent>>>,
    current: Mutex<Option<SessionId>>,
    state: Arc<RendererState>,
}

impl SessionReporter {
    /// A reporter that watches the given renderer state.
    #[must_use]
    pub fn new(state: Arc<RendererState>) -> Self {
        Self {
            events: Mutex::new(None),
            current: Mutex::new(None),
            state,
        }
    }

    /// Attaches the event channel of the engine.
    pub fn attach(&self, sender: tokio::sync::mpsc::Sender<SessionEvent>) {
        *self.events.lock().unwrap_or_else(|e| e.into_inner()) = Some(sender);
    }

    /// Detaches the event channel.
    pub fn detach(&self) {
        *self.events.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// The session that is currently reported.
    #[must_use]
    pub fn current(&self) -> Option<SessionId> {
        *self.current.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl EventSink for SessionReporter {
    fn publish(&self, _service_path: &str, last_change: &str) {
        let sender = self
            .events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let Some(sender) = sender else {
            return;
        };
        let mut current = self.current.lock().unwrap_or_else(|e| e.into_inner());
        let claimed = self.state.session().is_some();
        match (claimed, *current) {
            (true, None) => {
                let id = SessionId::next();
                let snapshot = SessionSnapshot {
                    id,
                    protocol: ProtocolKind::Dlna,
                    sender_name: None,
                    sender_address: None,
                    video: None,
                    audio: None,
                    started_at: std::time::Instant::now(),
                };
                if sender
                    .try_send(SessionEvent::Started { session: snapshot })
                    .is_ok()
                {
                    *current = Some(id);
                }
            }
            (false, Some(id)) => {
                *current = None;
                let _ = sender.try_send(SessionEvent::Ended {
                    session: id,
                    reason: SessionEndReason::SenderDisconnected,
                });
            }
            (true, Some(id)) => {
                let _ = sender.try_send(SessionEvent::PlaybackStateChanged {
                    session: id,
                    state: last_change.to_string(),
                });
            }
            (false, None) => {}
        }
    }
}
