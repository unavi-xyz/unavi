//! The signed claims a registry accepts: durable listings and live presence.

use iroh::EndpointId;
use iroh_docs::NamespaceId;
use serde::{
    Deserialize,
    Serialize,
};
use smol_str::SmolStr;
use unavi_identity::signed::Signable;
use xdid::core::did::Did;

use crate::error::RegistryError;

/// Longest [`Submission::title`], in bytes.
pub const MAX_TITLE_BYTES: usize = 128;
/// Longest [`Submission::description`], in bytes.
pub const MAX_DESCRIPTION_BYTES: usize = 1024;
/// Most [`Submission::tags`].
pub const MAX_TAGS: usize = 16;
/// Longest tag, in bytes.
pub const MAX_TAG_BYTES: usize = 32;
/// Most bytes a signed claim may encode to. Above every claim the field
/// limits allow, so it only bounds what is decoded before they are checked.
pub const MAX_CLAIM_BYTES: usize = 4096;

/// What a submission points at; a view slices by it without parsing the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Space,
    Avatar,
    Object,
}

/// A durable claim that a namespace is public and worth listing.
///
/// `did` must author `ns`, which the submitter proves alongside it (see
/// [`unavi_identity::authorship`]). Every other field is self-declared, and
/// therefore a trust input rather than a fact.
///
/// A blob hash may never become a field here: blob GC roots are entry *values*
/// only, not this struct's serialized content, so a hash carried inside would
/// name content nothing protects. Previews live under their own key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl Submission {
    /// Refuses fields past this crate's limits, which every client syncs.
    pub fn validate(&self) -> Result<(), RegistryError> {
        let too_long = self.title.len() > MAX_TITLE_BYTES
            || self
                .description
                .as_ref()
                .is_some_and(|text| text.len() > MAX_DESCRIPTION_BYTES)
            || self.tags.len() > MAX_TAGS
            || self.tags.iter().any(|tag| tag.len() > MAX_TAG_BYTES);

        if too_long {
            Err(RegistryError::TooLarge)
        } else {
            Ok(())
        }
    }
}

/// An ephemeral claim that a DID is reachable in a namespace right now, at
/// `endpoint`.
///
/// Never persisted: held in memory and expired by clock. A registry accepts it
/// only over a connection from `endpoint` itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presence {
    pub did:      Did,
    pub endpoint: EndpointId,
    pub ns:       NamespaceId,
    pub expires:  i64,
}

impl Signable for Presence {
    const SIGNING_CONTEXT: &'static str = "wired/registry/presence";
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn submission() -> Submission {
        Submission {
            did:         Did::from_str("did:web:example.com").expect("did"),
            ns:          NamespaceId::from(&[1; 32]),
            kind:        Kind::Space,
            title:       "a space".into(),
            description: None,
            tags:        vec!["social".into()],
            expires:     0,
        }
    }

    #[test]
    fn a_modest_listing_is_valid() {
        assert!(submission().validate().is_ok());
    }

    #[test]
    fn oversized_fields_are_refused() {
        let mut long_title = submission();
        long_title.title = "x".repeat(MAX_TITLE_BYTES + 1).into();

        let mut many_tags = submission();
        many_tags.tags = vec!["t".into(); MAX_TAGS + 1];

        for listing in [long_title, many_tags] {
            assert!(matches!(listing.validate(), Err(RegistryError::TooLarge)));
        }
    }
}
