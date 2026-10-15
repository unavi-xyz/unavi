//! The documents a registry publishes for clients to sync.

use iroh_docs::NamespaceId;
use serde::{
    Deserialize,
    Serialize,
};

/// Prefix of the active-spaces view. Each key is `active/{rank:08}/{ns}`, and
/// each value a postcard `(occupants: u32, idle_secs: u64)`.
pub const ACTIVE_PREFIX: &str = "active/";

/// Namespaces of the docs a registry publishes for clients to sync.
///
/// `recent` and `featured` hold `{rank:08}/{ns}` keys, `categories` holds
/// `{tag}/{rank:08}/{ns}` keys, each valued with a postcard
/// [`crate::claim::Submission`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ViewIds {
    pub recent:     NamespaceId,
    pub featured:   NamespaceId,
    pub categories: NamespaceId,
    pub active:     NamespaceId,
}

impl ViewIds {
    #[must_use]
    pub const fn all(&self) -> [NamespaceId; 4] {
        [self.recent, self.featured, self.categories, self.active]
    }
}
