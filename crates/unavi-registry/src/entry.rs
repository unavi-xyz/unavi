use iroh::EndpointId;
use iroh_docs::NamespaceId;
use serde::{
    Deserialize,
    Serialize,
};
use smol_str::SmolStr;
use unavi_identity::signed_bytes::Signable;
use xdid::core::did::Did;

/// What a submission points at; a view slices by it without parsing the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Space,
    Avatar,
    Object,
}

/// A durable claim that a namespace is public and worth listing.
///
/// Every field except `ns` and `did` is self-declared by the submitter and is
/// therefore a trust input, not a fact.
///
/// A blob hash may never become a field here: blob GC roots are entry *values*
/// only, not this struct's serialized content, so a hash carried inside would
/// name content nothing protects. Previews live under their own key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub did:         Did,
    pub ns:          NamespaceId,
    pub kind:        Kind,
    pub title:       SmolStr,
    pub description: Option<SmolStr>,
    pub tags:        Vec<SmolStr>,
    /// Unix timestamp after which the registry may drop this entry;
    /// resubmitting refreshes it.
    pub expires:     i64,
}

impl Signable for Submission {
    const SIGNING_CONTEXT: &'static str = "wired/registry/submission";
}

/// An ephemeral claim that a DID is reachable in a namespace right now.
///
/// Never persisted: held in memory and expired by clock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    pub did:      Did,
    pub endpoint: EndpointId,
    pub ns:       NamespaceId,
    pub expires:  i64,
}

impl Signable for Presence {
    const SIGNING_CONTEXT: &'static str = "wired/registry/presence";
}
