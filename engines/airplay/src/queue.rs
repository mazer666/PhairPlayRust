//! The bounded queues of the mirrored streams.
//!
//! WHY: the mirrored media arrives on a socket and is handed to the decoder
//!      of the platform. The decoder is *not* the receiver: it can be slower
//!      than the network, and a receiver that lets the two run free grows a
//!      buffer it does not control. The predecessor in Kotlin kept ninety
//!      frames in a queue that knew nothing but the count — ninety frames of
//!      eight megabytes are seven hundred and twenty megabytes, and one
//!      device on the local network can fill them.
//!
//! HOW: the frames travel through a **mailbox** that is bounded twice: by a
//!      number of frames *and* by a number of bytes. Both budgets come from
//!      the configuration of the receiver. When the budget is reached, the
//!      **oldest** frame is dropped — a stale picture is worth less than a
//!      fresh one — and the drop counter rises. The receiver never grows and
//!      it never blocks the reader.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::queue::{FrameQueue, Mailbox};
//! use phairplay_core::media::EncodedVideoFrame;
//! use phairplay_core::queue::{PushOutcome, QueueBudget, VideoFrameQueue};
//!
//! let mut queue = VideoFrameQueue::new(QueueBudget::new(2, 16));
//! let frame = EncodedVideoFrame::new(0, vec![1, 2, 3]);
//! assert_eq!(queue.push_frame(frame), PushOutcome::Stored);
//! ```

use std::sync::{Arc, Mutex};

use phairplay_core::media::{AudioSink, EncodedAudioFrame, EncodedVideoFrame, VideoSink};
use phairplay_core::queue::{AudioFrameQueue, PushOutcome, QueueBudget, VideoFrameQueue};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// A queue that knows its budget.
///
/// WHY: the mailbox needs to create the queue of the right kind, and it
///      cannot name the type in a generic function without this helper.
pub trait NewWithBudget: Sized {
    /// Creates an empty queue with the given budget.
    fn new_with_budget(budget: QueueBudget) -> Self;
}

impl NewWithBudget for VideoFrameQueue {
    fn new_with_budget(budget: QueueBudget) -> Self {
        VideoFrameQueue::new(budget)
    }
}

impl NewWithBudget for AudioFrameQueue {
    fn new_with_budget(budget: QueueBudget) -> Self {
        AudioFrameQueue::new(budget)
    }
}

/// The queue of a mailbox.
///
/// WHY the trait: the video frames and the audio packets travel through the
///      same machinery, and the only thing that differs is the payload and
///      the sink that receives it.
pub trait FrameQueue: NewWithBudget + Send + 'static {
    /// The frame that the queue carries.
    type Frame: Send + 'static;

    /// Puts a frame into the queue and reports what happened.
    fn push_frame(&mut self, frame: Self::Frame) -> PushOutcome;

    /// Takes the oldest frame out of the queue.
    fn pop_frame(&mut self) -> Option<Self::Frame>;

    /// The number of frames in the queue.
    fn queued(&self) -> usize;

    /// The number of bytes in the queue.
    fn queued_bytes(&self) -> usize;

    /// The number of frames that were dropped to keep the budget.
    fn dropped(&self) -> u64;
}

impl FrameQueue for VideoFrameQueue {
    type Frame = EncodedVideoFrame;

    fn push_frame(&mut self, frame: Self::Frame) -> PushOutcome {
        self.push(frame)
    }

    fn pop_frame(&mut self) -> Option<Self::Frame> {
        self.pop()
    }

    fn queued(&self) -> usize {
        self.len()
    }

    fn queued_bytes(&self) -> usize {
        self.bytes()
    }

    fn dropped(&self) -> u64 {
        self.dropped()
    }
}

impl FrameQueue for AudioFrameQueue {
    type Frame = EncodedAudioFrame;

    fn push_frame(&mut self, frame: Self::Frame) -> PushOutcome {
        self.push(frame)
    }

    fn pop_frame(&mut self) -> Option<Self::Frame> {
        self.pop()
    }

    fn queued(&self) -> usize {
        self.len()
    }

    fn queued_bytes(&self) -> usize {
        self.bytes()
    }

    fn dropped(&self) -> u64 {
        self.dropped()
    }
}

/// The sink that a mailbox delivers into.
pub trait FrameSink<Q: FrameQueue>: Clone + Send + Sync + 'static {
    /// Hands one frame to the decoder of the platform.
    fn deliver(&self, frame: Q::Frame);
}

impl FrameSink<VideoFrameQueue> for Arc<dyn VideoSink> {
    fn deliver(&self, frame: EncodedVideoFrame) {
        if let Err(error) = self.submit(frame) {
            tracing::warn!(%error, "the video sink rejected a frame");
        }
    }
}

impl FrameSink<AudioFrameQueue> for Arc<dyn AudioSink> {
    fn deliver(&self, frame: EncodedAudioFrame) {
        if let Err(error) = self.submit(frame) {
            tracing::warn!(%error, "the audio sink rejected a packet");
        }
    }
}

/// The bounded queue between the network and the sink of a mirrored stream.
///
/// WHY a mailbox and not a plain queue: the reader of the socket and the
///      decoder of the platform run in different tasks. The mailbox owns the
///      queue, the wake-up and the end-of-stream signal, so that the reader
///      never waits for the decoder and the decoder never waits for the
///      reader.
#[derive(Debug)]
pub struct Mailbox<Q: FrameQueue> {
    queue: Arc<Mutex<Q>>,
    data: mpsc::Sender<()>,
    close: mpsc::Sender<()>,
}

impl<Q: FrameQueue> Mailbox<Q> {
    /// Creates a mailbox and the task that empties it into the sink.
    ///
    /// WHY a task: the reader of the socket and the decoder of the platform
    ///      run in different tasks. The mailbox owns both ends, so that
    ///      neither of them waits for the other.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_airplay::queue::Mailbox;
    /// use phairplay_core::media::{NullVideoSink, VideoSink};
    /// use phairplay_core::queue::{QueueBudget, VideoFrameQueue};
    /// use std::sync::Arc;
    ///
    /// # let runtime = tokio::runtime::Runtime::new().expect("a runtime");
    /// # runtime.block_on(async {
    /// let sink: Arc<dyn VideoSink> = Arc::new(NullVideoSink);
    /// let (mailbox, delivery) =
    ///     Mailbox::<VideoFrameQueue>::spawn(sink, QueueBudget::new(8, 1024));
    /// mailbox.close();
    /// # let _ = delivery.await;
    /// # });
    /// ```
    pub fn spawn(sink: impl FrameSink<Q>, budget: QueueBudget) -> (Self, JoinHandle<()>) {
        let (data_sender, data_receiver) = mpsc::channel::<()>(1);
        let (close_sender, close_receiver) = mpsc::channel::<()>(1);
        let queue = Arc::new(Mutex::new(Q::new_with_budget(budget)));
        let delivery = tokio::spawn(deliver(
            Arc::clone(&queue),
            data_receiver,
            close_receiver,
            sink,
        ));
        (
            Self {
                queue,
                data: data_sender,
                close: close_sender,
            },
            delivery,
        )
    }

    /// Puts a frame into the queue.
    ///
    /// WHY the wake-up is not awaited: the channel holds one signal. A
    ///      signal that is already pending means that the consumer has not
    ///      emptied the queue yet, and it will look at the queue again when
    ///      it does. Waiting for the channel would block the reader of the
    ///      socket — the very thing the mailbox is there to prevent.
    pub fn push(&self, frame: Q::Frame) -> PushOutcome {
        let outcome = {
            let mut guard = self.queue.lock().unwrap_or_else(|error| error.into_inner());
            guard.push_frame(frame)
        };
        let _ = self.data.try_send(());
        outcome
    }

    /// The number of frames that are waiting in the queue.
    pub fn queued(&self) -> usize {
        let guard = self.queue.lock().unwrap_or_else(|error| error.into_inner());
        guard.queued()
    }

    /// The number of bytes that are waiting in the queue.
    pub fn queued_bytes(&self) -> usize {
        let guard = self.queue.lock().unwrap_or_else(|error| error.into_inner());
        guard.queued_bytes()
    }

    /// The number of frames that were dropped to keep the budget.
    pub fn dropped(&self) -> u64 {
        let guard = self.queue.lock().unwrap_or_else(|error| error.into_inner());
        guard.dropped()
    }

    /// Signals that no more frames will arrive.
    ///
    /// WHY: the consumer has to learn that the stream ended, otherwise it
    ///      would wait for a frame that never comes.
    pub fn close(&self) {
        let _ = self.close.try_send(());
    }
}

/// The task that empties a mailbox into the sink.
async fn deliver<Q: FrameQueue>(
    queue: Arc<Mutex<Q>>,
    mut data: mpsc::Receiver<()>,
    mut close: mpsc::Receiver<()>,
    sink: impl FrameSink<Q>,
) {
    loop {
        let frame = {
            let mut guard = queue.lock().unwrap_or_else(|error| error.into_inner());
            guard.pop_frame()
        };
        if let Some(frame) = frame {
            sink.deliver(frame);
            continue;
        }
        // The queue is empty: wait for the next frame or for the end of the
        // stream.
        tokio::select! {
            signal = data.recv() => {
                if signal.is_none() {
                    break;
                }
            }
            _ = close.recv() => break,
        }
    }
    // The stream ended: deliver what is left, so that the last frames of a
    // session are not lost.
    loop {
        let frame = {
            let mut guard = queue.lock().unwrap_or_else(|error| error.into_inner());
            guard.pop_frame()
        };
        match frame {
            Some(frame) => sink.deliver(frame),
            None => break,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use phairplay_core::media::EncodedVideoFrame;

    /// A sink that counts the frames it receives.
    #[derive(Debug, Default)]
    struct CountingSink {
        frames: Arc<Mutex<Vec<u64>>>,
    }

    impl FrameSink<VideoFrameQueue> for Arc<CountingSink> {
        fn deliver(&self, frame: EncodedVideoFrame) {
            let mut guard = self
                .frames
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard.push(frame.sequence);
        }
    }

    #[tokio::test]
    async fn the_frames_reach_the_sink() {
        let sink = Arc::new(CountingSink::default());
        let (mailbox, delivery) =
            Mailbox::<VideoFrameQueue>::spawn(sink.clone(), QueueBudget::new(8, 4096));
        for _ in 0..4 {
            mailbox.push(EncodedVideoFrame::new(0, vec![1, 2, 3]));
        }
        mailbox.close();
        delivery.await.expect("the delivery task finishes");
        let frames = sink
            .frames
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        assert_eq!(frames.len(), 4, "every frame is delivered");
    }

    #[tokio::test]
    async fn the_oldest_frame_is_dropped_when_the_budget_is_reached() {
        let sink = Arc::new(CountingSink::default());
        // Two frames of one kilobyte fit into a budget of two kilobytes.
        let (mailbox, delivery) =
            Mailbox::<VideoFrameQueue>::spawn(sink.clone(), QueueBudget::new(90, 2048));
        let mut pushed = Vec::new();
        for index in 0..4 {
            let frame = EncodedVideoFrame::new(0, vec![0_u8; 1024]);
            pushed.push(frame.sequence);
            let outcome = mailbox.push(frame);
            if index < 2 {
                assert_eq!(outcome, PushOutcome::Stored, "the first frames fit");
            } else {
                assert!(
                    matches!(outcome, PushOutcome::StoredAfterDrop { dropped: 1 }),
                    "the budget must drop the oldest frame: {outcome:?}"
                );
            }
        }
        assert!(mailbox.queued_bytes() <= 2048);
        assert_eq!(mailbox.dropped(), 2);
        mailbox.close();
        delivery.await.expect("the delivery task finishes");
        let frames = sink
            .frames
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        // The two newest frames survive; the two oldest are gone.
        assert_eq!(frames.len(), 2, "the frames that fit are delivered");
        assert!(frames.contains(&pushed[3]), "the newest frame is delivered");
        assert!(!frames.contains(&pushed[0]), "the oldest frame is dropped");
    }

    #[tokio::test]
    async fn a_frame_that_is_larger_than_the_budget_is_refused() {
        let sink = Arc::new(CountingSink::default());
        let (mailbox, delivery) =
            Mailbox::<VideoFrameQueue>::spawn(sink.clone(), QueueBudget::new(90, 1024));
        let outcome = mailbox.push(EncodedVideoFrame::new(0, vec![0_u8; 4096]));
        assert_eq!(outcome, PushOutcome::Rejected);
        assert_eq!(mailbox.queued(), 0);
        mailbox.close();
        delivery.await.expect("the delivery task finishes");
    }
}
