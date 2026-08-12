use iroh_docs::NamespaceId;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone)]
pub enum StateMsg {
    Snapshot(Vec<DocSnapshot>),
    Pin {
        doc:   NamespaceId,
        space: NamespaceId,
        /// Time the peer pinned; the oldest pin owns the document.
        at:    u64,
    },
    Unpin {
        doc: NamespaceId,
    },
    /// Transient transform authority over a document's rigid bodies (on grab);
    /// the latest claim wins, independent of ownership.
    Authority {
        doc:   NamespaceId,
        space: NamespaceId,
        at:    u64,
    },
    /// Releases the peer's authority claim over `doc`, falling authority back
    /// to the document's owner.
    Unclaim {
        doc: NamespaceId,
    },
    Kv {
        doc:   NamespaceId,
        space: NamespaceId,
        key:   String,
        value: Option<Vec<u8>>,
        at:    u64,
    },
    /// Drops the peer's own cell for `key` on a peer-owned document, without
    /// leaving a tombstone. Neutral (space-owned) cells are never forgotten
    /// this way; they outlive any single peer.
    KvForget {
        doc: NamespaceId,
        key: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocSnapshot {
    pub doc:       NamespaceId,
    pub space:     NamespaceId,
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
