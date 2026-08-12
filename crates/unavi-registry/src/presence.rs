use std::time::Duration;

use iroh_docs::NamespaceId;
use time::OffsetDateTime;
use wds::signed_bytes::SignedBytes;
use xdid::core::did::Did;

use crate::entry::Presence;

/// Live occupancy, held in memory and expired by clock.
#[derive(Default)]
pub struct PresenceTable {
    spaces: scc::HashMap<NamespaceId, Vec<Occupant>>,
}

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
    /// signed.
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
