use hsd::id::DocId;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone)]
pub enum StateMsg {
    Snapshot(Vec<DocSnapshot>),
    Pin {
        doc:   DocId,
        space: DocId,
        /// Time the peer pinned; the oldest pin owns the document.
        at:    u64,
    },
    Unpin {
        doc: DocId,
    },
    /// Takes hold of a document: transform and simulation authority over its
    /// rigid bodies (on grab). The latest claim wins, independent of who
    /// authored it.
    Hold {
        doc:   DocId,
        space: DocId,
        at:    u64,
    },
    /// Releases the peer's hold on `doc`, falling hold back to whoever owns
    /// it.
    ReleaseHold {
        doc: DocId,
    },
    /// Writes `key` on `doc`. A `value` of `None` is a tombstone, which is how
    /// a delete propagates — a cell belongs to the document, so a peer tearing
    /// down locally never tells anyone else to drop theirs.
    Kv {
        doc:   DocId,
        space: DocId,
        key:   String,
        value: Option<Vec<u8>>,
        at:    u64,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocSnapshot {
    pub doc:   DocId,
    pub space: DocId,
    /// Time the source peer pinned the doc, if it does.
    pub pin:   Option<u64>,
    /// When the source peer last took hold of the doc, if it holds it.
    pub hold:  Option<u64>,
    pub kv:    Vec<KvSnapshot>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KvSnapshot {
    pub key:   String,
    pub value: Option<Vec<u8>>,
    pub at:    u64,
}
