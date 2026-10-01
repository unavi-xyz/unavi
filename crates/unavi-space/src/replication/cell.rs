//! One session opinion and what reverting it leaves behind.

use hsd::{
    id::{
        DocId,
        PrimId,
    },
    property::name::PropName,
};
use iroh::EndpointId;
use unavi_policy::quota::StockLease;

/// Which key of which prim a session opinion is about.
///
/// The same `(prim, property)` key space the document uses, so an opinion a
/// peer states this session composes against what the document says — and a
/// commit can promote one into it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct SessionKey {
    pub(crate) prim: PrimId,
    pub(crate) name: PropName,
}

/// One session opinion, merged last-write-wins by `at`. `value: None` is a
/// retained tombstone, so a delete keeps winning over an older live write.
///
/// Cells live on the document, never under the peer that wrote one, so a cell
/// survives exactly as long as the document does, whichever of the author's
/// devices is present.
///
/// This is the record replication and revert read. The value also composes in
/// the document's own session layer, written by the same call that records it
/// here — one write, two places, because only this side knows who wrote it and
/// only that side can draw it.
pub(crate) struct Cell {
    pub(crate) at:    u64,
    pub(crate) peer:  EndpointId,
    pub(crate) value: Option<Vec<u8>>,
    pub(crate) lease: StockLease,
    /// Exactly one prior version, making "revert everything peer X wrote" a
    /// scan of the cell map rather than a general undo log.
    pub(crate) prev:  Option<Box<Self>>,
}

pub(crate) fn cell_bytes(key: &SessionKey, value: Option<&[u8]>) -> u64 {
    (key.name.as_str().len() + value.map_or(0, <[u8]>::len)) as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SessionError {
    #[error("session property name is invalid or too long")]
    BadName,
    /// The document has an author and the writer is not it, or the document
    /// belongs to another space.
    #[error("session write to an authored document by someone else")]
    NotOwner,
    #[error("session write exceeds quota")]
    QuotaExceeded,
    /// The world the replica lives in is gone.
    #[error("session write failed: replica unavailable")]
    Unavailable,
}

/// What stands on one key after a revert, which the document composing it has
/// to be told.
pub(crate) enum Standing {
    /// Nothing: the cell went, so the layers beneath it compose again.
    Gone,
    /// The prior writer's opinion, with its own stamp. A `value` of `None`
    /// blocks the key, as it did when they wrote it.
    Prior { value: Option<Vec<u8>>, at: u64 },
}

/// One key a revert touched.
pub(crate) struct Restored {
    pub(crate) doc:      DocId,
    pub(crate) key:      SessionKey,
    pub(crate) standing: Standing,
}
