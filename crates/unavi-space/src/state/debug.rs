//! Read-only views of the peer store for the dev tools state inspector.

use hsd::{
    id::{
        DocId,
        PrimId,
    },
    property::name::PropName,
};
use iroh::EndpointId;

/// One session opinion: what a peer said about one key of one prim.
pub struct DebugCell {
    pub prim:   PrimId,
    pub name:   PropName,
    /// The cell's value bytes; `None` is a blocked key.
    pub value:  Option<Vec<u8>>,
    pub at:     u64,
    pub writer: EndpointId,
}

/// What one peer contributes to a document.
pub struct DebugPeerDoc {
    pub doc:   DocId,
    pub space: DocId,
    pub pin:   Option<u64>,
    pub hold:  Option<u64>,
}

/// What a document holds regardless of which peer wrote it.
pub struct DebugDoc {
    pub doc:     DocId,
    pub space:   DocId,
    pub session: Vec<DebugCell>,
}

pub struct DebugPeer {
    pub peer: EndpointId,
    pub docs: Vec<DebugPeerDoc>,
}

pub struct DebugSnapshot {
    /// Each peer's pins and holds.
    pub peers: Vec<DebugPeer>,
    /// Session opinions, which are held by documents rather than by any peer.
    pub docs:  Vec<DebugDoc>,
}
