//! Read-only views of the peer store for the dev tools state inspector.

use hsd::id::DocId;
use iroh::EndpointId;

pub struct DebugKv {
    pub key:    String,
    /// The cell's value bytes; `None` is a tombstone.
    pub value:  Option<Vec<u8>>,
    pub at:     u64,
    pub writer: EndpointId,
}

/// What one peer contributes to a document.
pub struct DebugPeerDoc {
    pub doc:       DocId,
    pub space:     DocId,
    pub pin:       Option<u64>,
    pub authority: Option<u64>,
}

/// What a document holds regardless of which peer wrote it.
pub struct DebugDoc {
    pub doc:   DocId,
    pub space: DocId,
    pub kv:    Vec<DebugKv>,
}

pub struct DebugPeer {
    pub peer: EndpointId,
    pub docs: Vec<DebugPeerDoc>,
}

pub struct DebugSnapshot {
    /// Each peer's pins and authority claims.
    pub peers: Vec<DebugPeer>,
    /// KV, which is held by documents rather than by any peer.
    pub docs:  Vec<DebugDoc>,
}
