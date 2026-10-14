use serde::{
    Deserialize,
    Serialize,
};
use smol_str::SmolStr;

pub const DOC_VERSION: u16 = 2;

/// A document's own metadata, held at the reserved `meta/` key.
///
/// One blob rather than sub-keys: `meta/` prefixes anything below it, and the
/// format forbids data at a key that prefixes other keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocMeta {
    pub version:     u16,
    /// A display name. Self-declared, so a trust input rather than a fact.
    #[serde(default)]
    pub name:        Option<SmolStr>,
    #[serde(default)]
    pub description: Option<SmolStr>,
    /// A declaration that the author intends this document to be listed.
    /// The registry listing is the authoritative public fact.
    #[serde(default)]
    pub public:      bool,
    /// An opaque postcard `unavi_identity::authorship::Authorship`, kept as
    /// bytes so this crate stays free of identity concerns.
    #[serde(default)]
    pub author:      Option<Vec<u8>>,
}

impl Default for DocMeta {
    fn default() -> Self {
        Self {
            version:     DOC_VERSION,
            name:        None,
            description: None,
            public:      false,
            author:      None,
        }
    }
}
