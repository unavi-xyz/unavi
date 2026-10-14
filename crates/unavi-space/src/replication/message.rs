//! What peers tell each other over the state stream, and the one place a
//! received message is checked.

use hsd::id::{
    DocId,
    PrimId,
};
use iroh::EndpointId;
use serde::{
    Deserialize,
    Serialize,
};
use tracing::debug;
use unavi_identity::{
    authorship::Authorship,
    signed::{
        Signable,
        Signed,
    },
};
use xdid::{
    core::did::Did,
    method::key::Signer,
};

use crate::{
    membership::SpaceId,
    replication::{
        clock,
        record::{
            PinRecord,
            Record,
        },
        session::layers::{
            MAX_LAYERS_PER_DOC,
            RESERVED_LAYERS,
        },
    },
};

/// Longest property name a session write may carry. A name is a key, and a
/// key nobody can read back is not worth storing.
pub const SESSION_NAME_MAX_BYTES: usize = 256;

/// Most writes, and most clears, one session batch may carry.
pub const MAX_WRITES_PER_BATCH: usize = 256;

/// Longest value a session write may carry.
pub const MAX_VALUE_BYTES: usize = 16 * 1024;

/// Most documents one snapshot frame may describe.
pub const MAX_SNAPSHOT_DOCS: usize = 1024;

/// Most retained batches one snapshot layer may carry.
pub const MAX_BATCHES_PER_LAYER: usize = 1024;

#[derive(Serialize, Deserialize, Clone)]
pub enum ReplicationMsg {
    /// Everything the sender states, replacing whatever it stated before. A
    /// snapshot too large for one frame continues in [`Self::SnapshotMore`].
    Snapshot(Vec<DocSnapshot>),
    /// Further documents, or further layers of documents already sent, of the
    /// snapshot in progress.
    SnapshotMore(Vec<DocSnapshot>),
    /// The sender holds `doc` in `space`. `author` is the author's proof,
    /// whoever sends it; each receiver records whether the sender is the
    /// author. `record` is the newest record of the author's the sender
    /// holds, and says the document is pinned.
    Pin {
        doc:    DocId,
        space:  SpaceId,
        author: Authorship,
        record: Signed<PinRecord>,
    },
    /// The sender drops its pin. The author's last pinning device sends its
    /// withdrawal; a peer that drops its pin because of a withdrawal relays
    /// it.
    Unpin {
        doc:        DocId,
        withdrawal: Option<Signed<PinRecord>>,
    },
    /// Takes transform and simulation authority over `doc`'s rigid bodies.
    /// Honoured from the author, or from the peer the holder released to.
    Hold { doc: DocId, space: SpaceId },
    /// Drops the sender's hold on `doc`. With `to`, the sender, if it holds
    /// `doc`, lets that peer take hold next.
    ReleaseHold { doc: DocId, to: Option<EndpointId> },
    /// One batch of the sender's own session opinions.
    Session(Signed<SessionBatch>),
}

/// What a writer says about `doc`'s prims this session, as one atomic batch:
/// a tick's writes are applied together or not at all, so no peer ever draws
/// half of one. Signed by the writer's DID.
#[derive(Serialize, Deserialize, Clone)]
pub struct SessionBatch {
    /// The endpoint whose layer the batch belongs to.
    pub writer: EndpointId,
    pub did:    Did,
    pub doc:    DocId,
    pub space:  SpaceId,
    /// Microseconds, as the writer's clock reads.
    pub at:     u64,
    /// A `value` of `None` blocks the key, which is how a delete propagates.
    pub writes: Vec<SessionWrite>,
    /// Drops the writer's own opinions on these keys, so the layers beneath
    /// show through again. Sent after a commit makes them durable. A holder
    /// keeps each as a stamped tombstone, so an older write cannot fill the
    /// gap.
    pub clears: Vec<ClearedKey>,
}

impl Signable for SessionBatch {
    const SIGNING_CONTEXT: &'static str = "wired/space/session";
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClearedKey {
    pub prim: PrimId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionWrite {
    pub prim:  PrimId,
    pub name:  String,
    pub value: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocSnapshot {
    pub doc:        DocId,
    pub space:      SpaceId,
    /// The sender's own pin, if it pins the document.
    pub pin:        Option<PinSnapshot>,
    /// Whether the sender holds the document.
    pub hold:       bool,
    /// The newest withdrawal the sender knows for the document, listed even
    /// when the sender holds nothing else of it.
    pub withdrawal: Option<Signed<PinRecord>>,
    /// The layers the sender holds on the document, its own and the ones it
    /// carries for writers who left.
    pub layers:     Vec<LayerSnapshot>,
}

/// A pin as a snapshot states it: the author's proof, and the record the pin
/// was made with, which says the document is pinned.
#[derive(Serialize, Deserialize, Clone)]
pub struct PinSnapshot {
    pub author: Authorship,
    pub record: Signed<PinRecord>,
}

/// One writer's layer as the sender holds it.
#[derive(Serialize, Deserialize, Clone)]
pub struct LayerSnapshot {
    pub batches: Vec<RetainedBatch>,
}

/// A signed batch some cell or tombstone of the layer still points at.
#[derive(Serialize, Deserialize, Clone)]
pub struct RetainedBatch {
    pub batch:       Signed<SessionBatch>,
    /// Indices into the batch's `writes` that are still the writer's latest
    /// opinion on their key.
    pub live:        Vec<u16>,
    /// Indices into the batch's `clears` that are still the writer's latest
    /// word on their key.
    pub live_clears: Vec<u16>,
}

/// The parts of a batch a holder applies: indices into its `writes` and its
/// `clears`.
#[derive(Clone, Copy)]
pub struct Live<'a> {
    pub writes: &'a [u16],
    pub clears: &'a [u16],
}

impl RetainedBatch {
    #[must_use]
    pub fn live(&self) -> Live<'_> {
        Live {
            writes: &self.live,
            clears: &self.live_clears,
        }
    }
}

/// Why a received message was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Rejected {
    #[error("snapshot describes too many documents, layers or batches")]
    SnapshotTooLarge,
    #[error("session batch carries too many writes")]
    BatchTooLarge,
    #[error("session property name too long")]
    NameTooLong,
    #[error("session value too large")]
    ValueTooLarge,
    #[error("session batch is malformed")]
    Malformed,
    #[error("snapshot layer mixes writers, documents or spaces")]
    Mismatched,
}

/// A signed batch with its payload decoded once.
#[derive(Clone)]
pub struct Batch {
    pub signed:  Signed<SessionBatch>,
    pub payload: SessionBatch,
    /// The stamp storage and last-write-wins use. The signed `at` cannot be
    /// rewritten, so a received batch's is clamped here and `signed` relays
    /// the original.
    pub stamp:   u64,
}

impl Batch {
    /// Signs `payload` with the writer's DID key.
    pub fn sign(payload: SessionBatch, key: &impl Signer) -> anyhow::Result<Self> {
        let signed = payload.sign(key)?;
        Ok(Self {
            stamp: payload.at,
            signed,
            payload,
        })
    }

    fn received(signed: Signed<SessionBatch>, recv: u64) -> Result<Self, Rejected> {
        let payload = signed.payload().map_err(|_| Rejected::Malformed)?;
        if payload.writes.len() > MAX_WRITES_PER_BATCH
            || payload.clears.len() > MAX_WRITES_PER_BATCH
        {
            return Err(Rejected::BatchTooLarge);
        }
        for write in &payload.writes {
            check_name(&write.name)?;
            if write
                .value
                .as_ref()
                .is_some_and(|value| value.len() > MAX_VALUE_BYTES)
            {
                return Err(Rejected::ValueTooLarge);
            }
        }
        for clear in &payload.clears {
            check_name(&clear.name)?;
        }
        Ok(Self {
            stamp: clock::clamp_to(payload.at, recv),
            signed,
            payload,
        })
    }
}

/// A batch a snapshot relays, with the writes and clears of it that are still
/// current.
pub struct Retained {
    pub batch:       Batch,
    pub live:        Vec<u16>,
    pub live_clears: Vec<u16>,
}

impl Retained {
    #[must_use]
    pub fn live(&self) -> Live<'_> {
        Live {
            writes: &self.live,
            clears: &self.live_clears,
        }
    }
}

/// A [`PinSnapshot`] with its record decoded.
pub struct ReceivedPin {
    pub author: Authorship,
    pub record: Record,
}

/// A [`DocSnapshot`] whose batches are decoded and bounded.
pub struct ReceivedDoc {
    pub doc:        DocId,
    pub space:      SpaceId,
    pub pin:        Option<ReceivedPin>,
    pub hold:       bool,
    /// Decoded, not yet checked against the document's namespace.
    pub withdrawal: Option<Record>,
    /// Each layer's batches, all naming one writer and DID.
    pub layers:     Vec<Vec<Retained>>,
}

/// A [`ReplicationMsg`] that passed every bound. Records are decoded but not
/// yet checked against their document's namespace.
pub enum Received {
    Snapshot(Vec<ReceivedDoc>),
    SnapshotMore(Vec<ReceivedDoc>),
    Pin {
        doc:    DocId,
        space:  SpaceId,
        author: Authorship,
        record: Record,
    },
    Unpin {
        doc:        DocId,
        withdrawal: Option<Record>,
    },
    Hold {
        doc:   DocId,
        space: SpaceId,
    },
    ReleaseHold {
        doc: DocId,
        to:  Option<EndpointId>,
    },
    Session(Batch),
}

impl ReplicationMsg {
    /// Checks a message received at `recv` (micros) against every bound and
    /// decodes the batches it carries, clamping their stamps to no later than
    /// the allowed skew.
    pub fn validate(self, recv: u64) -> Result<Received, Rejected> {
        Ok(match self {
            Self::Snapshot(docs) => Received::Snapshot(check_docs(docs, recv)?),
            Self::SnapshotMore(docs) => Received::SnapshotMore(check_docs(docs, recv)?),
            Self::Session(signed) => Received::Session(Batch::received(signed, recv)?),
            Self::Pin {
                doc,
                space,
                author,
                record,
            } => Received::Pin {
                doc,
                space,
                author,
                record: Record::decode(record, recv)?,
            },
            Self::Unpin { doc, withdrawal } => Received::Unpin {
                doc,
                withdrawal: withdrawal
                    .map(|record| Record::decode(record, recv))
                    .transpose()?,
            },
            Self::Hold { doc, space } => Received::Hold { doc, space },
            Self::ReleaseHold { doc, to } => Received::ReleaseHold { doc, to },
        })
    }
}

fn check_docs(docs: Vec<DocSnapshot>, recv: u64) -> Result<Vec<ReceivedDoc>, Rejected> {
    if docs.len() > MAX_SNAPSHOT_DOCS {
        return Err(Rejected::SnapshotTooLarge);
    }
    docs.into_iter()
        .map(|doc| ReceivedDoc::check(doc, recv))
        .collect()
}

impl ReceivedDoc {
    fn check(snapshot: DocSnapshot, recv: u64) -> Result<Self, Rejected> {
        let DocSnapshot {
            doc,
            space,
            pin,
            hold,
            withdrawal,
            layers,
        } = snapshot;
        if layers.len() > MAX_LAYERS_PER_DOC + RESERVED_LAYERS {
            return Err(Rejected::SnapshotTooLarge);
        }
        // A layer that breaks its own bounds is skipped, so it cannot take the
        // document's pin, hold or other layers down with it.
        let layers = layers
            .into_iter()
            .filter_map(|layer| match check_layer(layer, doc, space, recv) {
                Ok(layer) => Some(layer),
                Err(err) => {
                    debug!(%doc, %err, "snapshot layer skipped");
                    None
                }
            })
            .collect();
        // Like a bad layer, an undecodable record costs the document only
        // that.
        let decode = |signed| match Record::decode(signed, recv) {
            Ok(record) => Some(record),
            Err(err) => {
                debug!(%doc, %err, "snapshot record skipped");
                None
            }
        };
        let pin = pin.and_then(|pin| {
            Some(ReceivedPin {
                author: pin.author,
                record: decode(pin.record)?,
            })
        });
        Ok(Self {
            doc,
            space,
            pin,
            hold,
            withdrawal: withdrawal.and_then(decode),
            layers,
        })
    }
}

fn check_layer(
    layer: LayerSnapshot,
    doc: DocId,
    space: SpaceId,
    recv: u64,
) -> Result<Vec<Retained>, Rejected> {
    if layer.batches.len() > MAX_BATCHES_PER_LAYER {
        return Err(Rejected::SnapshotTooLarge);
    }
    let mut retained = Vec::with_capacity(layer.batches.len());
    for RetainedBatch {
        batch,
        live,
        live_clears,
    } in layer.batches
    {
        let batch = Batch::received(batch, recv)?;
        let in_range = |live: &[u16], len: usize| {
            live.len() <= len && live.iter().all(|index| usize::from(*index) < len)
        };
        if !in_range(&live, batch.payload.writes.len())
            || !in_range(&live_clears, batch.payload.clears.len())
        {
            return Err(Rejected::Malformed);
        }
        let first: Option<&Retained> = retained.first();
        let same_layer = first.is_none_or(|first| {
            first.batch.payload.writer == batch.payload.writer
                && first.batch.payload.did == batch.payload.did
        });
        if batch.payload.doc != doc || batch.payload.space != space || !same_layer {
            return Err(Rejected::Mismatched);
        }
        retained.push(Retained {
            batch,
            live,
            live_clears,
        });
    }
    Ok(retained)
}

const fn check_name(name: &str) -> Result<(), Rejected> {
    if name.len() > SESSION_NAME_MAX_BYTES {
        Err(Rejected::NameTooLong)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use unavi_identity::identity::Identity;
    use xdid::method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    };

    use super::*;

    fn identity() -> Identity {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        Identity::new(did, key)
    }

    fn writer() -> EndpointId {
        iroh::SecretKey::from_bytes(&[5; 32]).public()
    }

    fn batch_of(who: &Identity, writer: EndpointId, at: u64, count: usize) -> Signed<SessionBatch> {
        SessionBatch {
            writer,
            did: who.did().clone(),
            doc: DocId([1; 32]),
            space: SpaceId([2; 32]),
            at,
            writes: (0..count)
                .map(|_| SessionWrite {
                    prim:  PrimId([3; 16]),
                    name:  "test/k".to_owned(),
                    value: None,
                })
                .collect(),
            clears: Vec::new(),
        }
        .sign(who.signing_key())
        .expect("sign")
    }

    fn snapshot(batches: Vec<RetainedBatch>) -> ReplicationMsg {
        ReplicationMsg::Snapshot(vec![DocSnapshot {
            doc:        DocId([1; 32]),
            space:      SpaceId([2; 32]),
            pin:        None,
            hold:       false,
            withdrawal: None,
            layers:     vec![LayerSnapshot { batches }],
        }])
    }

    fn clearing_batch(who: &Identity, count: usize) -> Signed<SessionBatch> {
        SessionBatch {
            writer: writer(),
            did:    who.did().clone(),
            doc:    DocId([1; 32]),
            space:  SpaceId([2; 32]),
            at:     0,
            writes: Vec::new(),
            clears: (0..count)
                .map(|_| ClearedKey {
                    prim: PrimId([3; 16]),
                    name: "test/k".to_owned(),
                })
                .collect(),
        }
        .sign(who.signing_key())
        .expect("sign")
    }

    #[test]
    fn a_future_stamp_is_clamped_to_the_skew_and_the_signed_stamp_is_kept() {
        let recv = 1_000_000_000;
        let alice = identity();
        let signed = batch_of(&alice, writer(), u64::MAX, 1);
        let Ok(Received::Session(batch)) = ReplicationMsg::Session(signed).validate(recv) else {
            panic!("valid batch");
        };
        assert_eq!(batch.stamp, recv + clock::MAX_SKEW_MICROS);
        assert_eq!(batch.payload.at, u64::MAX, "the signed stamp is relayed");
    }

    #[test]
    fn an_oversized_batch_is_rejected() {
        let signed = batch_of(&identity(), writer(), 0, MAX_WRITES_PER_BATCH + 1);
        assert_eq!(
            ReplicationMsg::Session(signed).validate(0).err(),
            Some(Rejected::BatchTooLarge)
        );
    }

    fn layers_of(msg: ReplicationMsg) -> Vec<usize> {
        let Ok(Received::Snapshot(docs)) = msg.validate(0) else {
            panic!("a bad layer does not reject the message");
        };
        docs[0].layers.iter().map(Vec::len).collect()
    }

    #[test]
    fn a_layer_breaking_a_bound_is_skipped_and_the_rest_of_the_snapshot_applies() {
        let alice = identity();
        let other = iroh::SecretKey::from_bytes(&[6; 32]).public();
        let good = RetainedBatch {
            batch:       batch_of(&alice, writer(), 0, 1),
            live:        vec![0],
            live_clears: Vec::new(),
        };
        let bad_index = RetainedBatch {
            batch:       batch_of(&alice, other, 0, 2),
            live:        vec![2],
            live_clears: Vec::new(),
        };
        let bad_clear_index = RetainedBatch {
            batch:       clearing_batch(&alice, 1),
            live:        Vec::new(),
            live_clears: vec![1],
        };
        let two_writers = [
            good.clone(),
            RetainedBatch {
                batch:       batch_of(&alice, other, 0, 1),
                live:        vec![0],
                live_clears: Vec::new(),
            },
        ];
        let too_many = (0..=MAX_BATCHES_PER_LAYER).map(|_| good.clone()).collect();
        let layer = |batches| LayerSnapshot { batches };
        let msg = ReplicationMsg::Snapshot(vec![DocSnapshot {
            doc:        DocId([1; 32]),
            space:      SpaceId([2; 32]),
            pin:        None,
            hold:       true,
            withdrawal: None,
            layers:     vec![
                layer(vec![bad_index]),
                layer(vec![bad_clear_index]),
                layer(two_writers.to_vec()),
                layer(too_many),
                layer(vec![good]),
            ],
        }]);
        let Ok(Received::Snapshot(docs)) = msg.validate(0) else {
            panic!("a bad layer does not reject the message");
        };
        assert!(docs[0].hold, "the document's hold survives");
        assert_eq!(docs[0].layers.len(), 1, "only the good layer remains");
    }

    #[test]
    fn a_snapshot_continuation_is_checked_like_a_snapshot() {
        let alice = identity();
        let msg = ReplicationMsg::SnapshotMore(vec![DocSnapshot {
            doc:        DocId([1; 32]),
            space:      SpaceId([2; 32]),
            pin:        None,
            hold:       false,
            withdrawal: None,
            layers:     vec![LayerSnapshot {
                batches: vec![RetainedBatch {
                    batch:       batch_of(&alice, writer(), 0, 1),
                    live:        vec![0],
                    live_clears: Vec::new(),
                }],
            }],
        }]);
        let Ok(Received::SnapshotMore(docs)) = msg.validate(0) else {
            panic!("a continuation validates");
        };
        assert_eq!(docs[0].layers.len(), 1);
    }

    #[test]
    fn a_layer_of_good_batches_is_kept() {
        let alice = identity();
        let msg = snapshot(vec![
            RetainedBatch {
                batch:       batch_of(&alice, writer(), 0, 2),
                live:        vec![1],
                live_clears: Vec::new(),
            },
            RetainedBatch {
                batch:       clearing_batch(&alice, 2),
                live:        Vec::new(),
                live_clears: vec![0, 1],
            },
        ]);
        assert_eq!(layers_of(msg), vec![2]);
    }

    #[test]
    fn a_value_past_the_cap_is_rejected() {
        let who = identity();
        let signed = SessionBatch {
            writer: writer(),
            did:    who.did().clone(),
            doc:    DocId([1; 32]),
            space:  SpaceId([2; 32]),
            at:     0,
            writes: vec![SessionWrite {
                prim:  PrimId([3; 16]),
                name:  "test/k".to_owned(),
                value: Some(vec![0; MAX_VALUE_BYTES + 1]),
            }],
            clears: Vec::new(),
        }
        .sign(who.signing_key())
        .expect("sign");
        assert_eq!(
            ReplicationMsg::Session(signed).validate(0).err(),
            Some(Rejected::ValueTooLarge)
        );
    }
}
