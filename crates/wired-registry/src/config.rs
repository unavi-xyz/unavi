use std::time::Duration;

use iroh_docs::NamespaceId;
use smol_str::SmolStr;
use xdid::core::did::Did;

/// Who may write to a registry.
#[derive(Debug, Clone, Default)]
pub enum Submitters {
    /// Any authenticated DID.
    #[default]
    Open,
    /// Only the listed DIDs.
    Allowlist(Vec<Did>),
}

/// Operator policy. None of this is protocol — it is where one registry
/// differs from another, which is the point of running your own.
#[derive(Debug, Clone)]
pub struct Config {
    pub submitters:              Submitters,
    /// Namespaces the operator promotes, regardless of ranking.
    pub featured:                Vec<NamespaceId>,
    /// Tags this registry recognizes as categories.
    pub categories:              Vec<SmolStr>,
    /// Maximum entries per view, bounding what a client must sync.
    pub view_capacity:           usize,
    /// How long after its last heartbeat a space still counts as active. Wider
    /// than the heartbeat interval, so a space does not drop out of discovery
    /// between one peer's announcements.
    pub activity_window:         Duration,
    /// Ceiling on how far ahead a submission may set its expiry.
    pub max_retention:           Duration,
    /// Abuse bound on catalog growth.
    pub max_submissions_per_did: usize,
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
