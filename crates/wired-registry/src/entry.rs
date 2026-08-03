use iroh_blobs::Hash;
use iroh_docs::NamespaceId;
use serde::{
    Deserialize,
    Serialize,
};
use smol_str::SmolStr;
use wds::signed_bytes::Signable;
use xdid::core::did::Did;

/// What a submission points at, so a view can slice by it without parsing the
/// target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Space,
    Avatar,
    Object,
}

/// A durable claim that a namespace is public and worth listing.
///
/// Every field except `ns` and `did` is self-declared by the submitter and is
/// therefore a trust input, not a fact. A registry weighs them at its own risk;
/// nothing here is verified beyond the signature and the announcer's identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub did:         Did,
    pub ns:          NamespaceId,
    pub kind:        Kind,
    pub title:       SmolStr,
    pub description: Option<SmolStr>,
    pub tags:        Vec<SmolStr>,
    pub preview:     Option<Hash>,
    /// Unix timestamp after which the registry may drop this entry. Refreshed
    /// by resubmitting, so abandoned submissions age out on their own.
    pub expires:     i64,
}

impl Signable for Submission {
    const SIGNING_CONTEXT: &'static str = "wired/registry/submission";
}

/// An ephemeral claim that a DID is reachable in a namespace right now.
///
/// Never persisted: presence is held in memory, expired by clock, and answered
/// by query. Writing it to a doc would spend a signed entry, a blob and a sync
/// fanout to record a fact that is false within minutes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    pub did:      Did,
    pub endpoint: [u8; 32],
    pub ns:       NamespaceId,
    pub expires:  i64,
}

impl Signable for Presence {
    const SIGNING_CONTEXT: &'static str = "wired/registry/presence";
}
