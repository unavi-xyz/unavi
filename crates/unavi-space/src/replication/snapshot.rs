//! Read-only copies of [`super::store::Replicas`], for inspection tools.

use hsd::{
    id::{
        DocId,
        PrimId,
    },
    property::name::PropName,
};
use iroh::EndpointId;

use crate::membership::SpaceId;

/// One session opinion: what a peer said about one key of one prim.
pub struct SessionCell {
    pub prim:   PrimId,
    pub name:   PropName,
    /// The cell's value bytes; `None` is a blocked key.
    pub value:  Option<Vec<u8>>,
    pub at:     u64,
    pub writer: EndpointId,
}

/// What one peer contributes to a document, stamped with local receive times.
pub struct PeerDoc {
    pub doc:   DocId,
    pub space: SpaceId,
    pub pin:   Option<u64>,
    pub hold:  Option<u64>,
}

/// What a document holds regardless of which peer wrote it.
pub struct DocState {
    pub doc:     DocId,
    pub space:   SpaceId,
    /// The DID its pin proved, if any.
    pub author:  Option<String>,
    pub session: Vec<SessionCell>,
}

pub struct PeerState {
    pub peer: EndpointId,
    pub docs: Vec<PeerDoc>,
}

pub struct ReplicaSnapshot {
    /// Each peer's pins and holds.
    pub peers: Vec<PeerState>,
    /// Every document present, with the session opinions it holds.
    pub docs:  Vec<DocState>,
}
