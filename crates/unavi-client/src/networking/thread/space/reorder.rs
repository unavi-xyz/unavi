//! P-frame reorder buffer for handling out-of-order UDP delivery.

use crate::networking::thread::space::msg::{AgentPFrameDatagram, ObjectPFrameDatagram};

/// Number of slots in the reorder buffer.
const REORDER_BUFFER_SIZE: usize = 3;

/// Trait for frames that can be reordered.
pub trait ReorderableFrame: Clone {
    fn iframe_id(&self) -> u16;
    fn seq(&self) -> u16;
}

impl ReorderableFrame for AgentPFrameDatagram {
    fn iframe_id(&self) -> u16 {
        self.iframe_id
    }
    fn seq(&self) -> u16 {
        self.seq
    }
}

impl ReorderableFrame for ObjectPFrameDatagram {
    fn iframe_id(&self) -> u16 {
        self.iframe_id
    }
    fn seq(&self) -> u16 {
        self.seq
    }
}

/// Reorder buffer for handling out-of-order P-frame delivery.
///
/// Buffers up to [`REORDER_BUFFER_SIZE`] frames to handle network reordering.
pub struct PFrameReorderBuffer<F: ReorderableFrame> {
    /// Current I-frame ID we're expecting P-frames for.
    iframe_id: u16,
    /// Last successfully applied sequence number.
    last_applied: u16,
    /// Pending frames waiting for earlier frames.
    pending: [Option<F>; REORDER_BUFFER_SIZE],
}

impl<F: ReorderableFrame> Default for PFrameReorderBuffer<F> {
    fn default() -> Self {
        Self {
            iframe_id: 0,
            last_applied: 0,
            pending: [const { None }; REORDER_BUFFER_SIZE],
        }
    }
}

impl<F: ReorderableFrame> PFrameReorderBuffer<F> {
    /// Reset buffer when a new I-frame arrives.
    pub fn reset(&mut self, new_iframe_id: u16) {
        self.iframe_id = new_iframe_id;
        self.last_applied = 0;
        self.pending = [const { None }; REORDER_BUFFER_SIZE];
    }

    /// Current I-frame ID this buffer is tracking.
    pub const fn iframe_id(&self) -> u16 {
        self.iframe_id
    }

    /// Process an incoming P-frame. Returns frames ready to apply in order.
    pub fn insert(&mut self, frame: F) -> ReadyFrames<F> {
        let mut ready = ReadyFrames::default();

        // Wrong I-frame window: discard.
        if frame.iframe_id() != self.iframe_id {
            return ready;
        }

        // Already applied or too old: discard.
        if frame.seq() <= self.last_applied && self.last_applied != 0 {
            return ready;
        }

        // First frame after reset: accept any seq as starting point.
        // Early P-frames may have been dropped due to baseline race.
        if self.last_applied == 0 {
            let seq = frame.seq();
            ready.push(frame);
            self.last_applied = seq;
            self.drain_consecutive(&mut ready);
            return ready;
        }

        let expected = self.last_applied.wrapping_add(1);

        if frame.seq() == expected {
            // In order: apply immediately.
            ready.push(frame);
            self.last_applied = expected;

            // Drain any consecutive pending frames.
            self.drain_consecutive(&mut ready);
        } else if frame.seq() > expected {
            // Out of order: check gap size.
            let gap = frame.seq().wrapping_sub(expected);

            if gap as usize <= REORDER_BUFFER_SIZE {
                // Buffer the frame.
                let slot = (frame.seq() as usize - 1) % REORDER_BUFFER_SIZE;
                self.pending[slot] = Some(frame);
            } else {
                // Gap too large: skip ahead.
                self.last_applied = frame.seq();
                ready.push(frame);
                self.pending = [const { None }; REORDER_BUFFER_SIZE];
            }
        }

        ready
    }

    fn drain_consecutive(&mut self, ready: &mut ReadyFrames<F>) {
        loop {
            let next = self.last_applied.wrapping_add(1);
            let slot = (next as usize - 1) % REORDER_BUFFER_SIZE;

            if let Some(frame) = self.pending[slot].take() {
                if frame.seq() == next {
                    ready.push(frame);
                    self.last_applied = next;
                    continue;
                }
                // Wrong sequence, put it back.
                self.pending[slot] = Some(frame);
            }
            break;
        }
    }
}

/// Collection of frames ready to be applied, in order.
pub struct ReadyFrames<F: ReorderableFrame> {
    frames: [Option<F>; REORDER_BUFFER_SIZE + 1],
    len: usize,
}

impl<F: ReorderableFrame> Default for ReadyFrames<F> {
    fn default() -> Self {
        Self {
            frames: [const { None }; REORDER_BUFFER_SIZE + 1],
            len: 0,
        }
    }
}

impl<F: ReorderableFrame> ReadyFrames<F> {
    fn push(&mut self, frame: F) {
        if self.len < self.frames.len() {
            self.frames[self.len] = Some(frame);
            self.len += 1;
        }
    }
}

impl<F: ReorderableFrame> Iterator for ReadyFrames<F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        // Shift frames down.
        let frame = self.frames[0].take();
        for i in 0..self.len - 1 {
            self.frames[i] = self.frames[i + 1].take();
        }
        self.len -= 1;
        frame
    }
}

#[cfg(test)]
mod tests {
    use half::f16;

    use super::*;
    use crate::networking::thread::space::types::{
        f16_pos::F16Pos,
        pose::{AgentPFrame, PFrameRootTransform},
        quat::PackedQuat,
    };

    fn make_frame(iframe_id: u16, seq: u16) -> AgentPFrameDatagram {
        AgentPFrameDatagram {
            iframe_id,
            seq,
            pose: AgentPFrame {
                root: PFrameRootTransform {
                    pos: F16Pos {
                        x: f16::ZERO,
                        y: f16::ZERO,
                        z: f16::ZERO,
                    },
                    rot: PackedQuat(0),
                },
                bones: Vec::new(),
            },
        }
    }

    #[test]
    fn test_in_order_delivery() {
        let mut buf = PFrameReorderBuffer::default();
        buf.reset(1);

        let ready: Vec<_> = buf.insert(make_frame(1, 1)).collect();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].seq, 1);

        let ready: Vec<_> = buf.insert(make_frame(1, 2)).collect();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].seq, 2);
    }

    #[test]
    fn test_first_frame_any_seq() {
        // First frame after reset can be any seq (earlier frames may have
        // been dropped due to baseline race).
        let mut buf = PFrameReorderBuffer::default();
        buf.reset(1);

        // First frame is seq=3 (1 and 2 were dropped).
        let ready: Vec<_> = buf.insert(make_frame(1, 3)).collect();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].seq, 3);

        // Next frame continues from there.
        let ready: Vec<_> = buf.insert(make_frame(1, 4)).collect();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].seq, 4);
    }

    #[test]
    fn test_single_out_of_order() {
        let mut buf = PFrameReorderBuffer::default();
        buf.reset(1);

        // Receive frame 1.
        let ready = buf.insert(make_frame(1, 1)).count();
        assert_eq!(ready, 1);

        // Skip frame 2, receive frame 3.
        let ready = buf.insert(make_frame(1, 3)).count();
        assert_eq!(ready, 0); // buffered

        // Now receive frame 2.
        let ready: Vec<_> = buf.insert(make_frame(1, 2)).collect();
        assert_eq!(ready.len(), 2);
        assert_eq!(ready[0].seq, 2);
        assert_eq!(ready[1].seq, 3);
    }

    #[test]
    fn test_large_gap_skip_ahead() {
        let mut buf = PFrameReorderBuffer::default();
        buf.reset(1);

        // Receive frame 1.
        let ready = buf.insert(make_frame(1, 1)).count();
        assert_eq!(ready, 1);

        // Skip many frames, receive frame 20.
        let ready: Vec<_> = buf.insert(make_frame(1, 20)).collect();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].seq, 20); // skipped ahead
    }

    #[test]
    fn test_iframe_reset() {
        let mut buf = PFrameReorderBuffer::default();
        buf.reset(1);

        // Receive frames for iframe 1.
        let ready = buf.insert(make_frame(1, 1)).count();
        assert_eq!(ready, 1);

        // Reset to new iframe.
        buf.reset(2);

        // Old iframe frames are rejected.
        let ready = buf.insert(make_frame(1, 2)).count();
        assert_eq!(ready, 0);

        // New iframe frames are accepted.
        let ready = buf.insert(make_frame(2, 1)).count();
        assert_eq!(ready, 1);
    }

    #[test]
    fn test_duplicate_rejection() {
        let mut buf = PFrameReorderBuffer::default();
        buf.reset(1);

        let ready = buf.insert(make_frame(1, 1)).count();
        assert_eq!(ready, 1);

        // Duplicate frame 1 should be rejected.
        let ready = buf.insert(make_frame(1, 1)).count();
        assert_eq!(ready, 0);
    }
}
