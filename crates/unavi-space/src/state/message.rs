use hsd::id::{
    DocId,
    PrimId,
};
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
    /// What the peer says about `doc`'s prims this session, as one atomic
    /// batch: a tick's writes are applied together or not at all, so no peer
    /// ever draws half of one.
    ///
    /// A `value` of `None` blocks the key, which is how a delete propagates —
    /// an opinion belongs to the document, so a peer tearing down locally
    /// never tells anyone else to drop theirs.
    Session {
        doc:    DocId,
        space:  DocId,
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
    pub space:   DocId,
    /// Time the source peer pinned the doc, if it does.
    pub pin:     Option<u64>,
    /// When the source peer last took hold of the doc, if it holds it.
    pub hold:    Option<u64>,
    pub session: Vec<SessionSnapshot>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionSnapshot {
    pub prim:  PrimId,
    pub name:  String,
    pub value: Option<Vec<u8>>,
    pub at:    u64,
}
