//! Sizing for the state stream: snapshots split across frames, and the
//! bounds of one session tick.

use tracing::warn;

use crate::replication::message::{
    DocSnapshot,
    ReplicationMsg,
    session::{
        LayerSnapshot,
        MAX_WRITES_PER_TICK,
        SessionWrite,
    },
};

/// Largest frame either side sends or accepts.
pub const MAX_STATE_FRAME: usize = 256 * 1024;
/// Most frames one snapshot may span, which bounds what a receiver buffers
/// before applying it.
pub const MAX_SNAPSHOT_FRAMES: usize = 256;

/// Most write content one tick carries, which keeps it well inside a frame.
pub(crate) const MAX_TICK_BYTES: usize = 64 * 1024;

/// Bytes a write takes encoded beyond its name and value: the prim id and the
/// length prefixes.
pub(crate) const WRITE_OVERHEAD_BYTES: usize = 24;

/// Encoded bytes a frame spends outside the documents it carries.
const FRAME_OVERHEAD_BYTES: usize = 16;

/// Encoded bytes of one write.
fn write_bytes(write: &SessionWrite) -> usize {
    WRITE_OVERHEAD_BYTES + write.name.len() + write.value.as_ref().map_or(0, Vec::len)
}

/// Whether `writes` fit one tick: at most [`MAX_WRITES_PER_TICK`] writes and
/// [`MAX_TICK_BYTES`] of content.
#[must_use]
pub fn fits_one_tick(writes: &[SessionWrite]) -> bool {
    writes.len() <= MAX_WRITES_PER_TICK
        && writes.iter().map(write_bytes).sum::<usize>() <= MAX_TICK_BYTES
}

/// The frames `msg` goes out as, none larger than `limit` encoded bytes. Only
/// a snapshot splits, and every frame of it but the last sets `more`. A cell
/// too large for any frame is skipped.
#[must_use]
pub fn split(msg: ReplicationMsg, limit: usize) -> Vec<ReplicationMsg> {
    let ReplicationMsg::Snapshot { scope, docs, .. } = msg else {
        return vec![msg];
    };
    let mut packer = Packer {
        budget:  limit.saturating_sub(FRAME_OVERHEAD_BYTES),
        frames:  Vec::new(),
        current: Vec::new(),
        size:    0,
    };
    for doc in docs {
        if let Err(doc) = packer.push(doc) {
            packer.split_doc(*doc);
        }
    }
    packer.flush();

    if packer.frames.is_empty() {
        packer.frames.push(Vec::new());
    }
    let last = packer.frames.len() - 1;
    packer
        .frames
        .into_iter()
        .enumerate()
        .map(|(index, docs)| ReplicationMsg::Snapshot {
            scope,
            docs,
            more: index < last,
        })
        .collect()
}

struct Packer {
    budget:  usize,
    frames:  Vec<Vec<DocSnapshot>>,
    current: Vec<DocSnapshot>,
    size:    usize,
}

impl Packer {
    /// Adds `doc` to the open frame, opening a new one if it does not fit.
    /// Hands the document back if it fits no frame.
    fn push(&mut self, doc: DocSnapshot) -> Result<(), Box<DocSnapshot>> {
        let Ok(len) = postcard::to_allocvec(&doc).map(|bytes| bytes.len()) else {
            return Err(Box::new(doc));
        };
        if len > self.budget {
            return Err(Box::new(doc));
        }
        if self.size + len > self.budget {
            self.flush();
        }
        self.size += len;
        self.current.push(doc);
        Ok(())
    }

    fn flush(&mut self) {
        if !self.current.is_empty() {
            self.frames.push(std::mem::take(&mut self.current));
            self.size = 0;
        }
    }

    /// Sends a document too large for a frame as its pin, hold and withdrawal,
    /// then each layer, splitting a layer too large for a frame by its cells.
    fn split_doc(&mut self, doc: DocSnapshot) {
        let DocSnapshot {
            doc,
            space,
            pin,
            hold,
            withdrawal,
            layers,
        } = doc;
        let piece = |layers| DocSnapshot {
            doc,
            space,
            pin: None,
            hold: None,
            withdrawal: None,
            layers,
        };
        let head = DocSnapshot {
            pin,
            hold,
            withdrawal,
            ..piece(Vec::new())
        };
        if (head.pin.is_some() || head.hold.is_some() || head.withdrawal.is_some())
            && self.push(head).is_err()
        {
            warn!(%doc, "pin, hold and withdrawal too large for a frame; skipped");
        }
        for layer in layers {
            self.push_layer(&piece, layer);
        }
    }

    /// Pushes `layer`, halving its cells until each piece fits a frame. Every
    /// piece repeats the layer's signed state.
    fn push_layer(
        &mut self,
        piece: &impl Fn(Vec<LayerSnapshot>) -> DocSnapshot,
        layer: LayerSnapshot,
    ) {
        let Err(whole) = self.push(piece(vec![layer])) else {
            return;
        };
        let Some(LayerSnapshot { state, mut cells }) = whole.layers.into_iter().next() else {
            return;
        };
        if cells.len() < 2 {
            warn!(doc = %whole.doc, "session cell too large for a frame; skipped");
            return;
        }
        let back = cells.split_off(cells.len() / 2);
        self.push_layer(
            piece,
            LayerSnapshot {
                state: state.clone(),
                cells,
            },
        );
        self.push_layer(piece, LayerSnapshot { state, cells: back });
    }
}

#[cfg(test)]
mod tests {
    use hsd::id::{
        DocId,
        PrimId,
    };
    use unavi_identity::identity::Identity;
    use xdid::method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    };

    use super::*;
    use crate::{
        membership::SpaceId,
        replication::message::session::{
            LayerState,
            SignedLayer,
        },
    };

    fn identity() -> Identity {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        Identity::new(did, key)
    }

    /// A document with one layer whose cells hold values of these sizes.
    fn doc_of(who: &Identity, seed: u8, values: Vec<usize>) -> DocSnapshot {
        let doc = DocId([seed; 32]);
        let space = SpaceId([2; 32]);
        let state = SignedLayer::sign(
            LayerState {
                device: iroh::SecretKey::from_bytes(&[5; 32]).public(),
                did: who.did().clone(),
                doc,
                space,
                at: 1,
                root: [0; 32],
            },
            who.signing_key(),
        )
        .expect("sign");
        let cells = values
            .into_iter()
            .enumerate()
            .map(|(n, len)| SessionWrite {
                prim:  PrimId([3; 16]),
                name:  format!("state/k{n}"),
                value: Some(vec![7; len]),
            })
            .collect();
        DocSnapshot {
            doc,
            space,
            pin: None,
            hold: None,
            withdrawal: None,
            layers: vec![LayerSnapshot {
                state: state.signed,
                cells,
            }],
        }
    }

    fn cells_in(frames: &[ReplicationMsg]) -> usize {
        frames
            .iter()
            .map(|frame| match frame {
                ReplicationMsg::Snapshot { docs, .. } => docs
                    .iter()
                    .flat_map(|doc| &doc.layers)
                    .map(|layer| layer.cells.len())
                    .sum::<usize>(),
                _ => 0,
            })
            .sum()
    }

    fn snapshot(docs: Vec<DocSnapshot>) -> ReplicationMsg {
        ReplicationMsg::Snapshot {
            scope: None,
            docs,
            more: false,
        }
    }

    #[test]
    fn a_small_snapshot_is_one_frame() {
        let who = identity();
        let frames = split(snapshot(vec![doc_of(&who, 1, vec![10])]), 4096);
        assert_eq!(frames.len(), 1);
        assert!(matches!(
            frames[0],
            ReplicationMsg::Snapshot { more: false, .. }
        ));
    }

    #[test]
    fn a_large_snapshot_continues_in_frames_that_each_fit() {
        let who = identity();
        let limit = 4096;
        let docs = (0..12)
            .map(|seed| doc_of(&who, seed, vec![1000, 1000]))
            .collect();
        let frames = split(snapshot(docs), limit);

        assert!(frames.len() > 1);
        let (last, rest) = frames.split_last().expect("frames");
        assert!(
            rest.iter()
                .all(|f| matches!(f, ReplicationMsg::Snapshot { more: true, .. })),
            "every frame but the last says more follows"
        );
        assert!(matches!(last, ReplicationMsg::Snapshot { more: false, .. }));
        for frame in &frames {
            let len = postcard::to_allocvec(frame).expect("encode").len();
            assert!(len <= limit, "a frame of {len} bytes exceeds {limit}");
        }
        assert_eq!(cells_in(&frames), 24, "nothing is dropped");
    }

    #[test]
    fn a_layer_too_large_for_a_frame_splits_by_cells() {
        let who = identity();
        let limit = 4096;
        let frames = split(
            snapshot(vec![doc_of(&who, 1, vec![1500, 1500, 1500, 1500])]),
            limit,
        );
        assert!(frames.len() > 1);
        for frame in &frames {
            assert!(postcard::to_allocvec(frame).expect("encode").len() <= limit);
        }
        assert_eq!(cells_in(&frames), 4);
    }

    #[test]
    fn a_cell_too_large_for_any_frame_is_skipped_and_the_rest_sent() {
        let who = identity();
        let frames = split(snapshot(vec![doc_of(&who, 1, vec![5000, 100])]), 2048);
        assert_eq!(cells_in(&frames), 1);
    }

    #[test]
    fn a_tick_fits_by_count_and_by_size() {
        let write = |len: usize| SessionWrite {
            prim:  PrimId([3; 16]),
            name:  "state/k".to_owned(),
            value: Some(vec![0; len]),
        };
        let many = (0..=MAX_WRITES_PER_TICK)
            .map(|_| write(1))
            .collect::<Vec<_>>();
        assert!(!fits_one_tick(&many));
        assert!(fits_one_tick(&many[1..]));
        assert!(!fits_one_tick(&[write(40_000), write(40_000)]));
    }
}
