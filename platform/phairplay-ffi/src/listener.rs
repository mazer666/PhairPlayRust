//! The listener that receives the live events of the receiver.
//!
//! WHAT: [`EventListener`] is the Rust trait every event consumer
//! implements. The UniFFI callback interface of the bridge and the
//! WebSocket channel of the dashboard are two adapters of it.
//!
//! WHY: The bridge must not know whether an event ends up in Kotlin, in
//! Swift or in a browser. One trait, many adapters — and the adapters stay
//! trivial enough to be obviously correct.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::FfiReceiverEvent;
//! use phairplay_ffi::listener::EventListener;
//!
//! #[derive(Debug, Default)]
//! struct Counter {
//!     seen: std::sync::atomic::AtomicUsize,
//! }
//!
//! impl EventListener for Counter {
//!     fn on_event(&self, _event: &FfiReceiverEvent) {
//!         self.seen.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
//!     }
//! }
//! ```

use std::sync::Arc;

use crate::callbacks::ReceiverEventSink;
use crate::dto::FfiReceiverEvent;

/// Receives the live events of the receiver.
pub trait EventListener: Send + Sync + std::fmt::Debug {
    /// Called for every event of the receiver.
    fn on_event(&self, event: &FfiReceiverEvent);
}

/// The adapter that hands the events of the bridge to a host.
///
/// WHY: The UniFFI callback interface hands out a `Box`; the supervisor
/// wants a shared handle that it can replace at any time. The adapter is
/// the bridge between the two.
pub struct CallbackEventListener {
    sink: Box<dyn ReceiverEventSink>,
}

impl std::fmt::Debug for CallbackEventListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackEventListener")
            .field("sink", &"<callback interface>")
            .finish()
    }
}

impl CallbackEventListener {
    /// Wraps the callback interface of a host.
    #[must_use]
    pub fn new(sink: Box<dyn ReceiverEventSink>) -> Arc<Self> {
        Arc::new(CallbackEventListener { sink })
    }
}

impl EventListener for CallbackEventListener {
    fn on_event(&self, event: &FfiReceiverEvent) {
        self.sink.on_event(event.clone());
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::dto::{FfiProtocolKind, FfiProtocolState, FfiProtocolStatus};
    use crate::test_support::Must;
    use std::sync::Mutex;

    #[derive(Debug, Default)]
    struct Recorder {
        events: Arc<Mutex<Vec<FfiReceiverEvent>>>,
    }

    impl ReceiverEventSink for Recorder {
        fn on_event(&self, event: FfiReceiverEvent) {
            if let Ok(mut events) = self.events.lock() {
                events.push(event);
            }
        }
    }

    fn status_event() -> FfiReceiverEvent {
        FfiReceiverEvent::ProtocolStatusChanged {
            status: FfiProtocolStatus {
                protocol: FfiProtocolKind::AirPlay,
                state: FfiProtocolState::Advertising,
                enabled: true,
                tcp_port: Some(7000),
                udp_port: None,
                peer_count: 0,
                last_error: None,
            },
        }
    }

    #[test]
    fn the_adapter_forwards_the_event_to_the_host() {
        let recorder = Arc::new(Recorder::default());
        let listener = CallbackEventListener::new(Box::new(Recorder {
            events: Arc::clone(&recorder.events),
        }));
        listener.on_event(&status_event());
        let events = recorder
            .events
            .lock()
            .must("the recorder must not be poisoned");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], status_event());
    }
}
