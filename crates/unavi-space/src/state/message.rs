use blake3::Hash;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone)]
pub enum StateMsg {
    Snapshot(Vec<DocSnapshot>),
    Pin {
        doc:   Hash,
        space: Hash,
        /// Time the peer pinned; the oldest pin owns the document.
        at:    u64,
    },
    Unpin {
        doc: Hash,
    },
    /// Transient transform authority over a document's rigid bodies (e.g. on
    /// grab); the latest claim wins, independent of ownership.
    Authority {
        doc:   Hash,
        space: Hash,
        at:    u64,
    },
    /// Releases the peer's authority claim over `doc`, falling authority back
    /// to the document's owner.
    Unclaim {
        doc: Hash,
    },
    Kv {
        doc:   Hash,
        space: Hash,
        key:   String,
        value: Option<Vec<u8>>,
        at:    u64,
    },
    /// Drops the peer's own cell for `key` on a peer-owned document, without
    /// leaving a tombstone. Neutral (space-owned) cells are never forgotten
    /// this way; they outlive any single peer.
    KvForget {
        doc: Hash,
        key: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocSnapshot {
    pub doc:       Hash,
    pub space:     Hash,
    /// Time the source peer pinned the doc, if it does.
    pub pin:       Option<u64>,
    /// The source peer's latest transform-authority claim, if any.
    pub authority: Option<u64>,
    pub kv:        Vec<KvSnapshot>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KvSnapshot {
    pub key:   String,
    pub value: Option<Vec<u8>>,
    pub at:    u64,
}
