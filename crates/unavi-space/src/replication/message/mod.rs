//! What peers tell each other over the state stream, and the one place a
//! received message is checked.

use std::collections::{
    HashMap,
    hash_map::Entry,
};

use hsd::id::DocId;
use serde::{
    Deserialize,
    Serialize,
};
use tracing::debug;
use unavi_identity::{
    authorship::Authorship,
    signed::Signed,
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

pub mod session;

use session::{
    LayerSnapshot,
    ReceivedLayer,
    ReceivedTick,
    SessionTick,
};

/// Most documents one snapshot frame may describe.
pub const MAX_SNAPSHOT_DOCS: usize = 1024;

#[derive(Serialize, Deserialize, Clone)]
pub enum ReplicationMsg {
    /// Everything the sender states, replacing whatever it stated before. With
    /// a `scope`, only what it states in that space is replaced, and every
    /// document must be in it. A snapshot too large for one frame is sent as
    /// consecutive frames, each but the last with `more` set, and is
    /// applied once the last arrives.
    Snapshot {
        scope: Option<SpaceId>,
        docs:  Vec<DocSnapshot>,
        more:  bool,
    },
    /// Asks the receiver for its state in `space`, which it answers with a
    /// snapshot scoped to that space. Sent by a peer that lost what it held of
    /// the receiver in `space`.
    Request { space: SpaceId },
    /// The sender holds `doc` in `space`. `author` is the author's proof,
    /// whoever sends it; each receiver records whether the sender is the
    /// author. `record` is the newest record of the author's the sender
    /// holds, and says the document is pinned.
    Pin {
        author: Authorship,
        record: Signed<PinRecord>,
    },
    /// The sender drops its pin. An author deleting the document sends its
    /// withdrawal; a peer that drops its pin because of a withdrawal relays
    /// it.
    Unpin {
        doc:        DocId,
        withdrawal: Option<Signed<PinRecord>>,
    },
    /// Takes transform and simulation authority over `doc`'s rigid bodies.
    /// `at` is the sender's clock in microseconds. The highest `(at, sender)`
    /// among every peer's holds resolves the holder, and a receiver refuses
    /// a hold stamped past its skew allowance.
    Hold(HoldClaim),
    /// Drops the sender's hold on `doc`.
    ReleaseHold { doc: DocId },
    /// One tick of the sender's own session opinions.
    Session(SessionTick),
}

/// A peer's claim to simulate `doc`'s rigid bodies, stamped `at` in
/// microseconds by its clock.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct HoldClaim {
    pub doc:   DocId,
    pub space: SpaceId,
    pub at:    u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocSnapshot {
    pub doc:        DocId,
    pub space:      SpaceId,
    /// The sender's own pin, if it pins the document.
    pub pin:        Option<PinSnapshot>,
    /// The stamp of the sender's hold on the document.
    pub hold:       Option<u64>,
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

/// Why a received message was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Rejected {
    #[error("snapshot describes too many documents or layers")]
    SnapshotTooLarge,
    #[error("session tick carries too many writes or bytes")]
    TickTooLarge,
    #[error("session property name too long")]
    NameTooLong,
    #[error("session value too large")]
    ValueTooLarge,
    #[error("message is malformed")]
    Malformed,
    #[error("hold stamped past the allowed clock skew")]
    FutureHold,
    #[error("snapshot layer names another document or space")]
    Mismatched,
}

/// A [`PinSnapshot`] with its record decoded.
pub struct ReceivedPin {
    pub author: Authorship,
    pub record: Record,
}

/// A [`DocSnapshot`] whose layers are decoded and bounded.
pub struct ReceivedDoc {
    pub doc:        DocId,
    pub space:      SpaceId,
    pub pin:        Option<ReceivedPin>,
    pub hold:       Option<u64>,
    /// Decoded, not yet checked against the document's namespace.
    pub withdrawal: Option<Record>,
    pub layers:     Vec<ReceivedLayer>,
}

/// A [`ReplicationMsg`] that passed every bound. Records are decoded but not
/// yet checked against their document's namespace.
pub enum Received {
    Snapshot {
        scope: Option<SpaceId>,
        docs:  Vec<ReceivedDoc>,
        more:  bool,
    },
    Request {
        space: SpaceId,
    },
    Pin {
        author: Authorship,
        record: Record,
    },
    Unpin {
        doc:        DocId,
        withdrawal: Option<Record>,
    },
    Hold(HoldClaim),
    ReleaseHold {
        doc: DocId,
    },
    Session(ReceivedTick),
}

impl ReplicationMsg {
    /// Checks a message received at `recv` (micros) against every bound and
    /// decodes what it carries.
    pub fn validate(self, recv: u64) -> Result<Received, Rejected> {
        Ok(match self {
            Self::Snapshot { scope, docs, more } => {
                if let Some(scope) = scope
                    && docs.iter().any(|doc| doc.space != scope)
                {
                    return Err(Rejected::Mismatched);
                }
                Received::Snapshot {
                    scope,
                    docs: check_docs(docs, recv)?,
                    more,
                }
            }
            Self::Request { space } => Received::Request { space },
            Self::Session(tick) => Received::Session(tick.check()?),
            Self::Pin { author, record } => Received::Pin {
                author,
                record: Record::decode(record, recv)?,
            },
            Self::Unpin { doc, withdrawal } => Received::Unpin {
                doc,
                withdrawal: withdrawal
                    .map(|record| Record::decode(record, recv))
                    .transpose()?,
            },
            Self::Hold(hold) => {
                if !clock::within_skew(hold.at, recv) {
                    return Err(Rejected::FutureHold);
                }
                Received::Hold(hold)
            }
            Self::ReleaseHold { doc } => Received::ReleaseHold { doc },
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
            .filter_map(|layer| match layer.check(doc, space) {
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
        let hold = hold.filter(|at| {
            let current = clock::within_skew(*at, recv);
            if !current {
                debug!(%doc, "snapshot hold stamped past the allowed skew; skipped");
            }
            current
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

    /// Folds in another piece of the same document from a later frame.
    fn merge(&mut self, piece: Self) {
        self.pin = self.pin.take().or(piece.pin);
        self.hold = self.hold.or(piece.hold);
        self.withdrawal = self.withdrawal.take().or(piece.withdrawal);
        for layer in piece.layers {
            let writer = layer.state.writer();
            match self
                .layers
                .iter_mut()
                .find(|held| held.state.writer() == writer)
            {
                Some(held) => held.merge(layer),
                None => self.layers.push(layer),
            }
        }
    }
}

/// Joins the pieces a snapshot split across frames back into one entry per
/// document, and one layer per writer.
#[must_use]
pub fn merge_snapshot(docs: Vec<ReceivedDoc>) -> Vec<ReceivedDoc> {
    let mut merged: HashMap<DocId, ReceivedDoc> = HashMap::new();
    for doc in docs {
        match merged.entry(doc.doc) {
            Entry::Occupied(mut held) if held.get().space == doc.space => held.get_mut().merge(doc),
            Entry::Occupied(_) => debug!(doc = %doc.doc, "snapshot piece names another space"),
            Entry::Vacant(slot) => {
                slot.insert(doc);
            }
        }
    }
    merged.into_values().collect()
}

#[cfg(test)] mod tests;
