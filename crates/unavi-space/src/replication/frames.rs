//! Sizing for the state stream: snapshots split across frames, and session
//! writes chunked into batches that fit one.

use tracing::warn;

use crate::replication::message::{
    DocSnapshot,
    LayerSnapshot,
    MAX_WRITES_PER_BATCH,
    ReplicationMsg,
    SessionWrite,
};

/// Largest frame either side sends or accepts.
pub const MAX_STATE_FRAME: usize = 256 * 1024;
/// Most frames one snapshot may span, which bounds what a receiver buffers
/// before applying it.
pub const MAX_SNAPSHOT_FRAMES: usize = 256;

/// Most write content one batch carries, which keeps a signed batch well
/// inside a frame.
pub(crate) const MAX_BATCH_BYTES: usize = 64 * 1024;

/// Bytes a write takes in an encoded batch beyond its name and value: the prim
/// id and the length prefixes.
pub(crate) const WRITE_OVERHEAD_BYTES: usize = 24;

/// Encoded bytes a frame spends outside the documents it carries.
const FRAME_OVERHEAD_BYTES: usize = 16;

/// Encoded bytes of one write.
fn write_bytes(write: &SessionWrite) -> usize {
    WRITE_OVERHEAD_BYTES + write.name.len() + write.value.as_ref().map_or(0, Vec::len)
}

/// Whether `writes` fit one batch: at most [`MAX_WRITES_PER_BATCH`] writes and
/// [`MAX_BATCH_BYTES`] of content.
#[must_use]
pub fn fits_one_batch(writes: &[SessionWrite]) -> bool {
    writes.len() <= MAX_WRITES_PER_BATCH
        && writes.iter().map(write_bytes).sum::<usize>() <= MAX_BATCH_BYTES
}

/// Splits `writes` into batches of at most [`MAX_WRITES_PER_BATCH`] writes and
/// [`MAX_BATCH_BYTES`] of content.
#[must_use]
pub fn chunk_writes(writes: Vec<SessionWrite>) -> Vec<Vec<SessionWrite>> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut bytes = 0;
    for write in writes {
        let size = write_bytes(&write);
        if !current.is_empty()
            && (current.len() >= MAX_WRITES_PER_BATCH || bytes + size > MAX_BATCH_BYTES)
        {
            chunks.push(std::mem::take(&mut current));
            bytes = 0;
        }
        bytes += size;
        current.push(write);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// The frames `msg` goes out as, none larger than `limit` encoded bytes. Only
/// a snapshot splits, and every frame of it but the last sets `more`. A batch
/// too large for any frame is skipped.
#[must_use]
pub fn split(msg: ReplicationMsg, limit: usize) -> Vec<ReplicationMsg> {
    let ReplicationMsg::Snapshot { docs, .. } = msg else {
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
    /// then each layer, then each batch of a layer too large for a frame.
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
            let Err(whole) = self.push(piece(vec![layer])) else {
                continue;
            };
            for batch in whole.layers.into_iter().flat_map(|layer| layer.batches) {
                let single = piece(vec![LayerSnapshot {
                    batches: vec![batch],
                }]);
                if self.push(single).is_err() {
                    warn!(%doc, "session batch too large for a frame; skipped");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hsd::id::{
        DocId,
        PrimId,
    };
    use unavi_identity::{
        identity::Identity,
        signed::Signable,
    };
    use xdid::method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    };

    use super::*;
    use crate::{
        membership::SpaceId,
        replication::message::{
            RetainedBatch,
            SessionBatch,
        },
    };

    fn retained(who: &Identity, doc: DocId, value: usize) -> RetainedBatch {
        let batch = SessionBatch {
            writer: iroh::SecretKey::from_bytes(&[5; 32]).public(),
            did: who.did().clone(),
            doc,
            space: SpaceId([2; 32]),
            at: 1,
            writes: vec![SessionWrite {
                prim:  PrimId([3; 16]),
                name:  "state/k".to_owned(),
                value: Some(vec![7; value]),
            }],
            clears: Vec::new(),
        };
        RetainedBatch {
            batch:       batch.sign(who.signing_key()).expect("sign"),
            live:        vec![0],
            live_clears: Vec::new(),
        }
    }

    fn doc_of(who: &Identity, seed: u8, batches: Vec<usize>) -> DocSnapshot {
        let doc = DocId([seed; 32]);
        DocSnapshot {
            doc,
            space: SpaceId([2; 32]),
            pin: None,
            hold: None,
            withdrawal: None,
            layers: vec![LayerSnapshot {
                batches: batches
                    .into_iter()
                    .map(|value| retained(who, doc, value))
                    .collect(),
            }],
        }
    }

    fn batches_in(frames: &[ReplicationMsg]) -> usize {
        frames
            .iter()
            .map(|frame| match frame {
                ReplicationMsg::Snapshot { docs, .. } => docs
                    .iter()
                    .flat_map(|doc| &doc.layers)
                    .map(|layer| layer.batches.len())
                    .sum::<usize>(),
                _ => 0,
            })
            .sum()
    }

    fn snapshot(docs: Vec<DocSnapshot>) -> ReplicationMsg {
        ReplicationMsg::Snapshot { docs, more: false }
    }

    fn identity() -> Identity {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        Identity::new(did, key)
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
        assert_eq!(batches_in(&frames), 24, "nothing is dropped");
    }

    #[test]
    fn a_document_too_large_for_a_frame_splits_by_layer_and_batch() {
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
        assert_eq!(batches_in(&frames), 4);
    }

    #[test]
    fn a_batch_too_large_for_any_frame_is_skipped_and_the_rest_sent() {
        let who = identity();
        let frames = split(snapshot(vec![doc_of(&who, 1, vec![5000, 100])]), 2048);
        assert_eq!(batches_in(&frames), 1);
    }

    #[test]
    fn writes_chunk_by_count_and_by_size() {
        let write = |len: usize| SessionWrite {
            prim:  PrimId([3; 16]),
            name:  "state/k".to_owned(),
            value: Some(vec![0; len]),
        };
        let by_count = chunk_writes((0..=MAX_WRITES_PER_BATCH).map(|_| write(1)).collect());
        assert_eq!(
            by_count.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![MAX_WRITES_PER_BATCH, 1]
        );

        let by_size = chunk_writes((0..6).map(|_| write(30_000)).collect());
        assert_eq!(by_size.len(), 3, "two 30 KB writes fill a 64 KB batch");
    }
}
