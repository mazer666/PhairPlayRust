//! The bounded media queues.
//!
//! WHY: a queue that is bounded by the *number of its entries* alone is not
//! bounded at all. A peer that sends ninety frames of eight megabytes fills
//! a queue of ninety entries with seven hundred megabytes, and the receiver
//! is killed by the out-of-memory killer — a denial of service that a single
//! device on the local network can trigger. The build contract therefore
//! demands **two** budgets for every queue: a number of entries *and* a
//! number of bytes.
//!
//! HOW: [`BoundedFrameQueue`] holds the entries in a FIFO and knows both
//! budgets. When an entry does not fit, the **oldest** entry is dropped — a
//! stale picture is worth less than a fresh one — and the drop counter rises.
//! An entry that is larger than the whole byte budget is refused outright:
//! it could never be stored, and storing it would break the budget that
//! protects the process.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::queue::{BoundedFrameQueue, PushOutcome, QueueBudget};
//!
//! let budget = QueueBudget { max_items: 4, max_bytes: 16 };
//! let mut queue = BoundedFrameQueue::new(budget);
//!
//! // Four entries of four bytes fit exactly.
//! for _ in 0..4 {
//!     assert_eq!(queue.push(1_u8, 4), PushOutcome::Stored);
//! }
//! assert_eq!(queue.len(), 4);
//!
//! // The fifth entry does not fit any more: the oldest one goes.
//! assert!(matches!(queue.push(2_u8, 4), PushOutcome::StoredAfterDrop { .. }));
//! assert_eq!(queue.len(), 4);
//! assert_eq!(queue.bytes(), 16);
//! ```

use std::collections::VecDeque;

use crate::config::LimitsConfig;

/// The two budgets of a media queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueBudget {
    /// The maximum number of entries in the queue.
    pub max_items: usize,
    /// The maximum number of bytes in the queue.
    pub max_bytes: usize,
}

impl QueueBudget {
    /// Creates a budget.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_core::queue::QueueBudget;
    ///
    /// let budget = QueueBudget { max_items: 90, max_bytes: 64 * 1024 * 1024 };
    /// assert_eq!(budget.max_items, 90);
    /// ```
    #[must_use]
    pub const fn new(max_items: usize, max_bytes: usize) -> Self {
        Self {
            max_items,
            max_bytes,
        }
    }

    /// The budget of the video queue, read from the configuration.
    ///
    /// WHY: the budget is never a constant in the code. It comes from the
    ///      configuration file the user can read, and the receiver refuses to
    ///      start with a budget of zero.
    #[must_use]
    pub fn video(limits: &LimitsConfig) -> Self {
        Self {
            max_items: limits.video_queue_frames,
            max_bytes: limits.video_queue_bytes,
        }
    }

    /// The budget of the audio queue, read from the configuration.
    #[must_use]
    pub fn audio(limits: &LimitsConfig) -> Self {
        Self {
            max_items: limits.audio_queue_packets,
            max_bytes: limits.audio_queue_bytes,
        }
    }
}

/// What happened to a pushed entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushOutcome {
    /// The entry was stored and nothing was dropped.
    Stored,
    /// The entry was stored, and older entries were dropped to make room.
    StoredAfterDrop {
        /// The number of entries that were dropped.
        dropped: usize,
    },
    /// The entry was refused: it is larger than the whole byte budget.
    Rejected,
}

/// A queue that is bounded by a number of entries *and* by a byte budget.
///
/// WHY: the two budgets are independent. A budget of ninety entries and a
///      budget of sixty-four megabytes protect two different resources: the
///      first the bookkeeping, the second the memory. A queue that only knows
///      the first one can be filled with hundreds of megabytes by a peer that
///      sends maximum-size frames.
#[derive(Debug)]
pub struct BoundedFrameQueue<T> {
    budget: QueueBudget,
    entries: VecDeque<(T, usize)>,
    bytes: usize,
    dropped: u64,
    rejected: u64,
}

impl<T> BoundedFrameQueue<T> {
    /// Creates an empty queue with the given budget.
    #[must_use]
    pub fn new(budget: QueueBudget) -> Self {
        Self {
            budget,
            entries: VecDeque::new(),
            bytes: 0,
            dropped: 0,
            rejected: 0,
        }
    }

    /// The budget of the queue.
    #[must_use]
    pub fn budget(&self) -> QueueBudget {
        self.budget
    }

    /// Puts an entry into the queue.
    ///
    /// WHY: the oldest entry is the one that goes. A queue of video frames
    ///      that drops the *newest* frame shows a frozen picture; a queue
    ///      that drops the oldest one shows a picture that is a little behind
    ///      — which is the lesser evil for a live stream.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_core::queue::{BoundedFrameQueue, PushOutcome, QueueBudget};
    ///
    /// let mut queue = BoundedFrameQueue::new(QueueBudget::new(2, 8));
    /// queue.push(1_u8, 4);
    /// queue.push(2_u8, 4);
    /// // The third entry does not fit: the first one is dropped.
    /// assert!(matches!(queue.push(3_u8, 4), PushOutcome::StoredAfterDrop { dropped: 1 }));
    /// assert_eq!(queue.pop(), Some(2));
    /// ```
    pub fn push(&mut self, value: T, size: usize) -> PushOutcome {
        // An entry that is larger than the whole budget could never be
        // stored. Storing it would break the budget that protects the
        // process, so it is refused and counted.
        if size > self.budget.max_bytes {
            self.rejected = self.rejected.saturating_add(1);
            return PushOutcome::Rejected;
        }
        let mut dropped = 0usize;
        while !self.entries.is_empty()
            && (self.entries.len() >= self.budget.max_items
                || self.bytes.saturating_add(size) > self.budget.max_bytes)
        {
            if let Some((_, old_size)) = self.entries.pop_front() {
                self.bytes = self.bytes.saturating_sub(old_size);
                dropped += 1;
            }
        }
        self.bytes = self.bytes.saturating_add(size);
        self.entries.push_back((value, size));
        if dropped > 0 {
            self.dropped = self.dropped.saturating_add(dropped as u64);
            PushOutcome::StoredAfterDrop { dropped }
        } else {
            PushOutcome::Stored
        }
    }

    /// Takes the oldest entry out of the queue.
    pub fn pop(&mut self) -> Option<T> {
        let (value, size) = self.entries.pop_front()?;
        self.bytes = self.bytes.saturating_sub(size);
        Some(value)
    }

    /// The number of entries in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The number of bytes in the queue.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    /// The number of entries that were dropped to keep the budget.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    /// The number of entries that were refused because they were too large.
    #[must_use]
    pub fn rejected(&self) -> u64 {
        self.rejected
    }

    /// Removes every entry and counts the bytes that are freed.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }
}

mod frames;

pub use frames::{AudioFrameQueue, VideoFrameQueue};

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::media::EncodedVideoFrame;

    /// A frame of `size` bytes.
    fn frame(size: usize) -> EncodedVideoFrame {
        EncodedVideoFrame::new(0, vec![0_u8; size])
    }

    #[test]
    fn a_queue_starts_empty() {
        let queue: BoundedFrameQueue<u8> = BoundedFrameQueue::new(QueueBudget::new(4, 16));
        assert!(queue.is_empty());
        assert_eq!(queue.bytes(), 0);
        assert_eq!(queue.dropped(), 0);
    }

    #[test]
    fn entries_are_returned_in_the_order_they_arrived() {
        let mut queue = BoundedFrameQueue::new(QueueBudget::new(4, 16));
        queue.push(1_u8, 1);
        queue.push(2_u8, 1);
        queue.push(3_u8, 1);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn the_oldest_entry_is_dropped_when_the_item_budget_is_reached() {
        let mut queue = BoundedFrameQueue::new(QueueBudget::new(2, 1024));
        queue.push(1_u8, 1);
        queue.push(2_u8, 1);
        let outcome = queue.push(3_u8, 1);
        assert_eq!(outcome, PushOutcome::StoredAfterDrop { dropped: 1 });
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.dropped(), 1);
    }

    #[test]
    fn the_oldest_entry_is_dropped_when_the_byte_budget_is_reached() {
        let mut queue = BoundedFrameQueue::new(QueueBudget::new(90, 16));
        queue.push(1_u8, 8);
        queue.push(2_u8, 8);
        // The third entry of eight bytes does not fit into sixteen bytes:
        // the oldest entry goes, and the budget holds.
        let outcome = queue.push(3_u8, 8);
        assert_eq!(outcome, PushOutcome::StoredAfterDrop { dropped: 1 });
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.bytes(), 16);
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
    }

    #[test]
    fn an_entry_that_is_larger_than_the_budget_is_refused() {
        let mut queue = BoundedFrameQueue::new(QueueBudget::new(90, 16));
        assert_eq!(queue.push(1_u8, 17), PushOutcome::Rejected);
        assert!(queue.is_empty());
        assert_eq!(queue.rejected(), 1);
    }

    #[test]
    fn the_byte_budget_holds_under_the_ninety_frames_of_eight_megabytes_attack() {
        // WHY this shape: the predecessor in Kotlin kept ninety frames in a
        // queue that knew nothing but the count. A peer that sends ninety
        // frames of eight megabytes therefore filled seven hundred and twenty
        // megabytes into the process. The same attack is executed here with
        // one-megabyte frames, so that the test stays cheap while the ratio
        // stays the same: ninety frames against a budget of four megabytes.
        let budget = QueueBudget::new(90, 4 * 1024 * 1024);
        let mut queue = BoundedFrameQueue::new(budget);
        let mut pushed = Vec::new();
        for _ in 0..90 {
            let frame = frame(1024 * 1024);
            pushed.push(frame.sequence);
            queue.push(frame, 1024 * 1024);
            // The budget must hold after *every* push, not only at the end.
            assert!(
                queue.bytes() <= budget.max_bytes,
                "the queue held {} bytes",
                queue.bytes()
            );
        }
        assert!(queue.bytes() <= budget.max_bytes);
        assert!(queue.len() <= budget.max_items);
        assert!(queue.dropped() > 0, "the attack did not drop anything");
        // The entries that are left are the newest ones: the first frames
        // that were pushed are gone.
        let first_survivor = queue
            .pop()
            .map(|frame| frame.sequence)
            .expect("the queue still holds frames");
        assert!(
            first_survivor > pushed[0],
            "the oldest frame survived the attack: {first_survivor}"
        );
    }

    #[test]
    fn the_newest_entries_survive_the_attack() {
        let budget = QueueBudget::new(90, 4 * 1024 * 1024);
        let mut queue = VideoFrameQueue::new(budget);
        let mut last = None;
        for _ in 0..90 {
            let frame = frame(1024 * 1024);
            last = Some(frame.sequence);
            queue.push(frame);
        }
        let newest = last.expect("a frame was pushed");
        // The queue holds the newest frames; the oldest ones are gone.
        let mut survivors = Vec::new();
        while let Some(frame) = queue.pop() {
            survivors.push(frame.sequence);
        }
        assert!(survivors.contains(&newest), "the newest frame was dropped");
        assert!(survivors.len() <= 90);
    }

    #[test]
    fn the_budget_comes_from_the_configuration() {
        let limits = LimitsConfig {
            video_queue_frames: 7,
            video_queue_bytes: 1024,
            audio_queue_packets: 3,
            audio_queue_bytes: 2048,
            ..LimitsConfig::default()
        };
        assert_eq!(QueueBudget::video(&limits), QueueBudget::new(7, 1024));
        assert_eq!(QueueBudget::audio(&limits), QueueBudget::new(3, 2048));
    }

    #[test]
    fn clearing_a_queue_frees_the_bytes() {
        let mut queue: BoundedFrameQueue<u8> = BoundedFrameQueue::new(QueueBudget::new(4, 16));
        queue.push(1_u8, 8);
        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.bytes(), 0);
    }
}
