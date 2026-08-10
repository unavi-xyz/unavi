use std::time::Duration;

use iroh_docs::NamespaceId;
use time::OffsetDateTime;
use wds::signed_bytes::SignedBytes;
use xdid::core::did::Did;

use crate::entry::Presence;

/// Live occupancy, held in memory and expired by clock.
///
/// Keyed by namespace so the common query — who is in this space — is a single
/// lookup rather than a scan.
#[derive(Default)]
pub struct PresenceTable {
    spaces: scc::HashMap<NamespaceId, Vec<Occupant>>,
}

/// A space with recent activity.
pub struct ActiveSpace {
    pub ns:        NamespaceId,
    pub occupants: usize,
    pub idle_secs: u64,
}

struct Occupant {
    did:       Did,
    expires:   i64,
    last_seen: i64,
    signed:    SignedBytes<Presence>,
}

impl PresenceTable {
    /// Records a heartbeat, replacing any previous one from the same DID.
    pub async fn insert(&self, presence: &Presence, signed: SignedBytes<Presence>) {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let did = presence.did.clone();
        let expires = presence.expires;

        self.spaces
            .entry_async(presence.ns)
            .await
            .and_modify(|occupants| {
                occupants.retain(|o| o.did != did);
                occupants.push(Occupant {
                    did: did.clone(),
                    expires,
                    last_seen: now,
                    signed: signed.clone(),
                });
            })
            .or_insert_with(|| {
                vec![Occupant {
                    did,
                    expires,
                    last_seen: now,
                    signed,
                }]
            });
    }

    /// Returns the unexpired occupants of a namespace, still individually
    /// signed so the caller verifies rather than trusting this registry.
    pub async fn occupants(&self, ns: NamespaceId) -> Vec<SignedBytes<Presence>> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.spaces
            .read_async(&ns, |_, occupants| {
                occupants
                    .iter()
                    .filter(|o| o.expires > now)
                    .map(|o| o.signed.clone())
                    .collect()
            })
            .await
            .unwrap_or_default()
    }

    /// Spaces active within `window`, most recently active first.
    ///
    /// Lookup by namespace answers "who is here"; this answers "where is
    /// anyone", which is what discovery needs and cannot be derived from the
    /// former without already knowing every namespace.
    ///
    /// Reporting over a window rather than instantaneous occupancy is what
    /// keeps the answer stable: heartbeats arrive minutes apart, so an exact
    /// snapshot would flicker as peers lapse and renew.
    pub async fn active(&self, window: Duration) -> Vec<ActiveSpace> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let cutoff = now - i64::try_from(window.as_secs()).unwrap_or(i64::MAX);
        let mut out = Vec::new();

        self.spaces
            .retain_async(|ns, occupants| {
                let seen = occupants.iter().filter(|o| o.last_seen > cutoff);
                let occupant_count = seen.clone().count();
                if let Some(last_seen) = seen.map(|o| o.last_seen).max() {
                    out.push(ActiveSpace {
                        ns:        *ns,
                        occupants: occupant_count,
                        idle_secs: u64::try_from(now - last_seen).unwrap_or(0),
                    });
                }
                true
            })
            .await;

        out.sort_by_key(|s| s.idle_secs);
        out
    }

    /// Drops occupants that have not been heard from within `window`.
    pub async fn sweep(&self, window: Duration) {
        let cutoff = OffsetDateTime::now_utc().unix_timestamp()
            - i64::try_from(window.as_secs()).unwrap_or(i64::MAX);
        self.spaces
            .retain_async(|_, occupants| {
                occupants.retain(|o| o.last_seen > cutoff);
                !occupants.is_empty()
            })
            .await;
    }
}
