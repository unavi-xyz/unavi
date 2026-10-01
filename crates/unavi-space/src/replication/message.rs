//! What peers tell each other over the state stream, and the one place a
//! received message is checked.

use hsd::id::{
    DocId,
    PrimId,
};
use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::authorship::Authorship;

use crate::{
    membership::SpaceId,
    replication::clock,
};

/// Longest property name a session write may carry. A name is a key, and a
/// key nobody can read back is not worth storing.
pub const SESSION_NAME_MAX_BYTES: usize = 256;

/// Most writes one session batch may carry.
pub const MAX_WRITES_PER_BATCH: usize = 256;

/// Most documents one snapshot may describe.
pub const MAX_SNAPSHOT_DOCS: usize = 1024;

/// Most session cells one snapshot document may carry.
pub const MAX_SNAPSHOT_CELLS: usize = 1024;

#[derive(Serialize, Deserialize, Clone)]
pub enum ReplicationMsg {
    /// Everything the sender states, replacing whatever it stated before.
    Snapshot(Vec<DocSnapshot>),
    /// The sender is present with `doc` in `space`, and proves it authors it.
    /// Only a document's author may pin it.
    Pin {
        doc:    DocId,
        space:  SpaceId,
        author: Authorship,
    },
    Unpin {
        doc: DocId,
    },
    /// Takes transform and simulation authority over `doc`'s rigid bodies.
    /// Honoured from the author, or from the peer the holder released to.
    Hold {
        doc:   DocId,
        space: SpaceId,
    },
    /// Drops the sender's hold on `doc`. With `to`, the sender, if it holds
    /// `doc`, lets that peer take hold next.
    ReleaseHold {
        doc: DocId,
        to:  Option<iroh::EndpointId>,
    },
    /// What the peer says about `doc`'s prims this session, as one atomic
    /// batch: a tick's writes are applied together or not at all, so no peer
    /// ever draws half of one.
    ///
    /// A `value` of `None` blocks the key, which is how a delete propagates.
    /// An opinion belongs to the document, so a peer tearing down locally
    /// never tells anyone else to drop theirs.
    Session {
        doc:    DocId,
        space:  SpaceId,
        writes: Vec<SessionWrite>,
        at:     u64,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionWrite {
    pub prim:  PrimId,
    pub name:  String,
    pub value: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocSnapshot {
    pub doc:     DocId,
    pub space:   SpaceId,
    /// The sender's authorship proof, if it pins the document.
    pub pin:     Option<Authorship>,
    /// Whether the sender holds the document.
    pub hold:    bool,
    pub session: Vec<SessionSnapshot>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionSnapshot {
    pub prim:  PrimId,
    pub name:  String,
    pub value: Option<Vec<u8>>,
    pub at:    u64,
}

/// Why a received message was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Rejected {
    #[error("snapshot describes too many documents")]
    SnapshotTooLarge,
    #[error("session batch carries too many writes")]
    BatchTooLarge,
    #[error("session property name too long")]
    NameTooLong,
}

impl ReplicationMsg {
    /// The spaces this message speaks for.
    pub fn spaces(&self) -> impl Iterator<Item = SpaceId> + '_ {
        let (one, many) = match self {
            Self::Snapshot(docs) => (None, Some(docs.iter().map(|d| d.space))),
            Self::Pin { space, .. } | Self::Hold { space, .. } | Self::Session { space, .. } => {
                (Some(*space), None)
            }
            Self::Unpin { .. } | Self::ReleaseHold { .. } => (None, None),
        };
        one.into_iter().chain(many.into_iter().flatten())
    }

    /// Checks a message received at `recv` (micros) against every bound,
    /// clamping its session stamps to no later than the allowed skew.
    pub fn validate(mut self, recv: u64) -> Result<Self, Rejected> {
        match &mut self {
            Self::Snapshot(docs) => {
                if docs.len() > MAX_SNAPSHOT_DOCS {
                    return Err(Rejected::SnapshotTooLarge);
                }
                for doc in docs {
                    if doc.session.len() > MAX_SNAPSHOT_CELLS {
                        return Err(Rejected::SnapshotTooLarge);
                    }
                    for cell in &mut doc.session {
                        check_name(&cell.name)?;
                        cell.at = clock::clamp_to(cell.at, recv);
                    }
                }
            }
            Self::Session { writes, at, .. } => {
                if writes.len() > MAX_WRITES_PER_BATCH {
                    return Err(Rejected::BatchTooLarge);
                }
                for write in writes.iter() {
                    check_name(&write.name)?;
                }
                *at = clock::clamp_to(*at, recv);
            }
            Self::Pin { .. }
            | Self::Unpin { .. }
            | Self::Hold { .. }
            | Self::ReleaseHold { .. } => {}
        }
        Ok(self)
    }
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
    use super::*;

    fn session(at: u64, writes: usize) -> ReplicationMsg {
        ReplicationMsg::Session {
            doc: DocId([1; 32]),
            space: SpaceId([2; 32]),
            writes: (0..writes)
                .map(|_| SessionWrite {
                    prim:  PrimId([3; 16]),
                    name:  "test/k".to_owned(),
                    value: None,
                })
                .collect(),
            at,
        }
    }

    #[test]
    fn a_future_stamp_is_clamped_to_the_skew() {
        let recv = 1_000_000_000;
        let Ok(ReplicationMsg::Session { at, .. }) = session(u64::MAX, 1).validate(recv) else {
            panic!("valid batch");
        };
        assert_eq!(at, recv + clock::MAX_SKEW_MICROS);
    }

    #[test]
    fn an_oversized_batch_is_rejected() {
        assert_eq!(
            session(0, MAX_WRITES_PER_BATCH + 1).validate(0).err(),
            Some(Rejected::BatchTooLarge)
        );
    }
}
