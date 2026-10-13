//! Operator policy. None of this is protocol.

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

/// Operator policy.
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
    /// Abuse bound on one identity's share of the catalog.
    pub max_submissions_per_did: usize,
    /// Abuse bound on the whole catalog.
    pub max_listings:            usize,
    /// Most namespaces one DID may be present in at once.
    pub max_presences_per_did:   usize,
    /// Abuse bound on the listings all `did:key` identities hold together,
    /// since minting one costs nothing.
    pub max_did_key_listings:    usize,
    /// Abuse bound on the presences all `did:key` identities hold together.
    pub max_did_key_presences:   usize,
    /// Abuse bound on occupants across every namespace.
    pub max_presences:           usize,
    /// Most devices of one DID present in one namespace.
    pub max_devices_per_did:     usize,
    /// Most occupants one namespace lists.
    pub max_occupants:           usize,
    /// Ceiling on how far ahead a presence may set its expiry.
    pub max_presence_ttl:        Duration,
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
            max_submissions_per_did: 8,
            max_listings:            10_000,
            max_presences_per_did:   4,
            max_did_key_listings:    1_000,
            max_did_key_presences:   2_000,
            max_presences:           50_000,
            max_devices_per_did:     4,
            max_occupants:           256,
            max_presence_ttl:        Duration::from_mins(10),
        }
    }
}

/// Whether `did` costs nothing to mint, and so shares a pooled allowance.
pub fn is_did_key(did: &Did) -> bool {
    did.method_name.as_str() == "key"
}

impl Config {
    /// Whether `did` may submit at all.
    #[must_use]
    pub fn permits(&self, did: &Did) -> bool {
        match &self.submitters {
            Submitters::Open => true,
            Submitters::Allowlist(allowed) => allowed.contains(did),
        }
    }
}
