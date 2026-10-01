//! The bounded queue behind every subscription.

use std::collections::VecDeque;

use parking_lot::Mutex;

/// Items a subscription may fall behind by. A script drains once per tick, so
/// reaching this means it has stopped reading.
pub const DEPTH: usize = 64;

/// Most items one `drain` returns.
pub const MAX_DRAIN: u32 = 256;

/// A queue written by the host and drained by a script. Overflow drops the
/// oldest item and counts it: a reader this far behind wants what just
/// happened more than what happened a second ago.
pub struct Queue<T>(Mutex<Inner<T>>);

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self(Mutex::new(Inner::default()))
    }
}

struct Inner<T> {
    items:   VecDeque<T>,
    dropped: u64,
}

impl<T> Default for Inner<T> {
    fn default() -> Self {
        Self {
            items:   VecDeque::new(),
            dropped: 0,
        }
    }
}

impl<T> Queue<T> {
    pub fn push(&self, item: T) {
        let mut inner = self.0.lock();
        if inner.items.len() >= DEPTH {
            inner.items.pop_front();
            inner.dropped = inner.dropped.saturating_add(1);
        }
        inner.items.push_back(item);
    }

    /// Up to `max` items, oldest first, capped at [`MAX_DRAIN`].
    pub fn drain(&self, max: u32) -> Vec<T> {
        let mut inner = self.0.lock();
        let n = inner.items.len().min(max.min(MAX_DRAIN) as usize);
        inner.items.drain(..n).collect()
    }

    /// Items lost to overflow since the queue was created.
    pub fn dropped(&self) -> u64 {
        self.0.lock().dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_come_back_in_the_order_they_were_pushed() {
        let queue = Queue::default();
        queue.push(1);
        queue.push(2);
        assert_eq!(queue.drain(8), vec![1, 2]);
        assert_eq!(queue.drain(8).len(), 0);
    }

    #[test]
    fn a_full_queue_keeps_the_newest_items_and_counts_the_rest() {
        let queue = Queue::default();
        for i in 0..=DEPTH {
            queue.push(i);
        }
        let held = queue.drain(u32::MAX);
        assert_eq!(held.len(), DEPTH, "the queue stays bounded");
        assert_eq!(held.last(), Some(&DEPTH), "the newest item is kept");
        assert_eq!(queue.dropped(), 1);
    }

    #[test]
    fn a_drain_takes_at_most_what_was_asked() {
        let queue = Queue::default();
        for i in 0..4 {
            queue.push(i);
        }
        assert_eq!(queue.drain(3), vec![0, 1, 2]);
        assert_eq!(queue.drain(3), vec![3]);
    }
}
