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
    },
    Unpin {
        doc: Hash,
    },
    Claim {
        doc: Hash,
        at:  u64,
    },
    Kv {
        doc:   Hash,
        space: Hash,
        key:   String,
        value: Option<Vec<u8>>,
        at:    u64,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocSnapshot {
    pub doc:    Hash,
    pub space:  Hash,
    /// Whether the source peer pins this doc for instancing, as opposed to
    /// merely holding KV state for it.
    pub pinned: bool,
    pub claim:  Option<u64>,
    pub kv:     Vec<KvSnapshot>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KvSnapshot {
    pub key:   String,
    pub value: Option<Vec<u8>>,
    pub at:    u64,
}
