// WHAT: the two media queues of the core: one for the video frames and one
//      for the audio packets.
// WHY: both queues are the same bounded queue with a different payload. The
//      payload decides how many bytes an entry occupies, and that is the only
//      difference between them.
// EXAMPLE: `VideoFrameQueue::from_limits` reads the budget of the video
//          queue out of the configuration of the receiver.

use crate::config::LimitsConfig;
use crate::media::{EncodedAudioFrame, EncodedVideoFrame};
use crate::queue::{BoundedFrameQueue, PushOutcome, QueueBudget};

/// A queue of encoded video frames.
#[derive(Debug)]
pub struct VideoFrameQueue {
    inner: BoundedFrameQueue<EncodedVideoFrame>,
}

impl VideoFrameQueue {
    /// Creates an empty queue with the given budget.
    #[must_use]
    pub fn new(budget: QueueBudget) -> Self {
        Self {
            inner: BoundedFrameQueue::new(budget),
        }
    }

    /// Creates an empty queue with the budget of the configuration.
    #[must_use]
    pub fn from_limits(limits: &LimitsConfig) -> Self {
        Self::new(QueueBudget::video(limits))
    }

    /// Puts a frame into the queue.
    pub fn push(&mut self, frame: EncodedVideoFrame) -> PushOutcome {
        let size = frame.data.len();
        self.inner.push(frame, size)
    }

    /// Takes the oldest frame out of the queue.
    pub fn pop(&mut self) -> Option<EncodedVideoFrame> {
        self.inner.pop()
    }

    /// The number of frames in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// The number of bytes in the queue.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.inner.bytes()
    }

    /// The number of frames that were dropped to keep the budget.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.inner.dropped()
    }

    /// Removes every frame.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

/// A queue of encoded audio packets.
#[derive(Debug)]
pub struct AudioFrameQueue {
    inner: BoundedFrameQueue<EncodedAudioFrame>,
}

impl AudioFrameQueue {
    /// Creates an empty queue with the given budget.
    #[must_use]
    pub fn new(budget: QueueBudget) -> Self {
        Self {
            inner: BoundedFrameQueue::new(budget),
        }
    }

    /// Creates an empty queue with the budget of the configuration.
    #[must_use]
    pub fn from_limits(limits: &LimitsConfig) -> Self {
        Self::new(QueueBudget::audio(limits))
    }

    /// Puts a packet into the queue.
    pub fn push(&mut self, packet: EncodedAudioFrame) -> PushOutcome {
        let size = packet.data.len();
        self.inner.push(packet, size)
    }

    /// Takes the oldest packet out of the queue.
    pub fn pop(&mut self) -> Option<EncodedAudioFrame> {
        self.inner.pop()
    }

    /// The number of packets in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// The number of bytes in the queue.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.inner.bytes()
    }

    /// The number of packets that were dropped to keep the budget.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.inner.dropped()
    }

    /// Removes every packet.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
