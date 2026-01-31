//! P-frame reorder buffer for handling out-of-order UDP delivery.

use super::PFrameDatagram;

/// Number of slots in the reorder buffer.
const REORDER_BUFFER_SIZE: usize = 3;

/// Reorder buffer for handling out-of-order P-frame delivery.
///
/// Buffers up to [`REORDER_BUFFER_SIZE`] frames to handle network reordering.
#[derive(Default)]
pub struct PFrameReorderBuffer {
    /// Current I-frame ID we're expecting P-frames for.
    iframe_id: u16,
    /// Last successfully applied sequence number.
    last_applied: u16,
    /// Pending frames waiting for earlier frames.
    pending: [Option<PFrameDatagram>; REORDER_BUFFER_SIZE],
}

impl PFrameReorderBuffer {
    /// Reset buffer when a new I-frame arrives.
    pub fn reset(&mut self, new_iframe_id: u16) {
        self.iframe_id = new_iframe_id;
        self.last_applied = 0;
        self.pending = Default::default();
    }

    /// Current I-frame ID this buffer is tracking.
    pub const fn iframe_id(&self) -> u16 {
        self.iframe_id
    }

    /// Process an incoming P-frame. Returns frames ready to apply in order.
    pub fn insert(&mut self, frame: PFrameDatagram) -> ReadyFrames {
        let mut ready = ReadyFrames::default();

        // Wrong I-frame window: discard.
        if frame.iframe_id != self.iframe_id {
            return ready;
        }

        // Already applied or too old: discard.
        if frame.seq <= self.last_applied && self.last_applied != 0 {
            return ready;
        }

        // Special case: first frame after reset.
        if self.last_applied == 0 && frame.seq == 1 {
            ready.push(frame);
            self.last_applied = 1;
            self.drain_consecutive(&mut ready);
            return ready;
        }

        let expected = self.last_applied.wrapping_add(1);

        if frame.seq == expected {
            // In order: apply immediately.
            ready.push(frame);
            self.last_applied = expected;

            // Drain any consecutive pending frames.
            self.drain_consecutive(&mut ready);
        } else if frame.seq > expected {
            // Out of order: check gap size.
            let gap = frame.seq.wrapping_sub(expected);

            if gap as usize <= REORDER_BUFFER_SIZE {
                // Buffer the frame.
                let slot = (frame.seq as usize - 1) % REORDER_BUFFER_SIZE;
                self.pending[slot] = Some(frame);
            } else {
                // Gap too large: skip ahead.
                self.last_applied = frame.seq;
                ready.push(frame);
                self.pending = Default::default();
            }
        }

        ready
    }

    fn drain_consecutive(&mut self, ready: &mut ReadyFrames) {
        loop {
            let next = self.last_applied.wrapping_add(1);
            let slot = (next as usize - 1) % REORDER_BUFFER_SIZE;

            if let Some(frame) = self.pending[slot].take() {
                if frame.seq == next {
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
#[derive(Default)]
pub struct ReadyFrames {
    frames: [Option<PFrameDatagram>; REORDER_BUFFER_SIZE + 1],
    len: usize,
}

impl ReadyFrames {
    fn push(&mut self, frame: PFrameDatagram) {
        if self.len < self.frames.len() {
            self.frames[self.len] = Some(frame);
            self.len += 1;
        }
    }
}

impl Iterator for ReadyFrames {
    type Item = PFrameDatagram;

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
    use super::*;
    use crate::networking::thread::space::{
        pos::PFramePos,
        pose::{PFrameTransform, PlayerPFrame},
        quat::PackedQuat,
    };

    fn make_frame(iframe_id: u16, seq: u16) -> PFrameDatagram {
        PFrameDatagram {
            iframe_id,
            seq,
            pose: PlayerPFrame {
                root: PFrameTransform {
                    pos: PFramePos { x: 0, y: 0, z: 0 },
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
