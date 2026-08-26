use std::time::Duration;

use iroh_docs::NamespaceId;
use serde::{
    Deserialize,
    Serialize,
};
use smol_str::SmolStr;
use xdid::core::did::Did;

use crate::views::ViewIds;

/// The namespaces backing one registry's durable documents.
///
/// Minted by the registry when absent and persisted by its operator, so ids
/// clients already sync still name the same docs after a restart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryDocs {
    pub catalog: NamespaceId,
    pub views:   ViewIds,
}

/// Who may write to a registry.
#[derive(Debug, Clone, Default)]
pub enum Submitters {
    /// Any authenticated DID.
    #[default]
    Open,
    /// Only the listed DIDs.
    Allowlist(Vec<Did>),
}

/// Operator policy; none of this is protocol.
#[derive(Debug, Clone)]
pub struct Config {
    pub submitters:              Submitters,
    /// Namespaces the operator promotes, regardless of ranking.
    pub featured:                Vec<NamespaceId>,
    /// Tags this registry recognizes as categories.
    pub categories:              Vec<SmolStr>,
    /// Maximum entries per view, bounding what a client must sync.
    pub view_capacity:           usize,
    /// How long after its last heartbeat a space still counts as active.
    /// Wider than the heartbeat interval.
    pub activity_window:         Duration,
    /// Ceiling on how far ahead a submission may set its expiry.
    pub max_retention:           Duration,
    /// Abuse bound on catalog growth.
    pub max_submissions_per_did: usize,
    /// Docs this registry already minted, from operator state. Absent on a
    /// fresh deployment; the registry mints then and reports the ids back.
    pub docs:                    Option<RegistryDocs>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            submitters:              Submitters::Open,
            featured:                Vec::new(),
            categories:              Vec::new(),
            view_capacity:           256,
            activity_window:         Duration::from_mins(5),
            max_retention:           Duration::from_hours(24 * 30),
            max_submissions_per_did: 64,
            docs:                    None,
        }
    }
}

impl Config {
    #[must_use]
    pub fn permits(&self, did: &Did) -> bool {
        match &self.submitters {
            Submitters::Open => true,
            Submitters::Allowlist(allowed) => allowed.contains(did),
        }
    }
}
