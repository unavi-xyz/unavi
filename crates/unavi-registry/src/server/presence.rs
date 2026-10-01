//! Live occupancy, held in memory, bounded per DID and per namespace, and
//! expired by clock.

use std::{
    collections::HashMap,
    time::Duration,
};

use iroh::EndpointId;
use iroh_docs::NamespaceId;
use parking_lot::Mutex;
use time::OffsetDateTime;
use unavi_identity::signed::Signed;
use xdid::core::did::Did;

use crate::{
    claim::Presence,
    error::RegistryError,
    server::config::Config,
};

#[derive(Default)]
pub struct PresenceTable(Mutex<Inner>);

#[derive(Default)]
struct Inner {
    spaces: HashMap<NamespaceId, Vec<Occupant>>,
    /// Devices each DID has present, per namespace.
    by_did: HashMap<Did, HashMap<NamespaceId, usize>>,
}

pub struct ActiveSpace {
    pub ns:        NamespaceId,
    pub occupants: usize,
    pub idle_secs: u64,
}

struct Occupant {
    did:       Did,
    endpoint:  EndpointId,
    expires:   i64,
    last_seen: i64,
    signed:    Signed<Presence>,
}

impl PresenceTable {
    /// Records a heartbeat, replacing any previous one from the same endpoint.
    pub fn insert(
        &self,
        presence: &Presence,
        signed: Signed<Presence>,
        config: &Config,
    ) -> Result<(), RegistryError> {
        let now = now();
        let mut inner = self.0.lock();
        inner.expire_did(&presence.did, now);

        let (spaces_held, devices_here) = inner.by_did.get(&presence.did).map_or((0, 0), |held| {
            (held.len(), held.get(&presence.ns).copied().unwrap_or(0))
        });
        let occupants = inner.spaces.get(&presence.ns);
        let refreshing =
            occupants.is_some_and(|list| list.iter().any(|o| o.endpoint == presence.endpoint));

        if !refreshing {
            if devices_here == 0 && spaces_held >= config.max_presences_per_did {
                return Err(RegistryError::TooManyPresences);
            }
            if devices_here >= config.max_devices_per_did {
                return Err(RegistryError::TooManyPresences);
            }
            if occupants.is_some_and(|list| list.len() >= config.max_occupants) {
                return Err(RegistryError::NamespaceFull);
            }
        }

        inner.remove(presence.ns, |o| o.endpoint == presence.endpoint);
        inner.spaces.entry(presence.ns).or_default().push(Occupant {
            did: presence.did.clone(),
            endpoint: presence.endpoint,
            expires: presence.expires,
            last_seen: now,
            signed,
        });
        *inner
            .by_did
            .entry(presence.did.clone())
            .or_default()
            .entry(presence.ns)
            .or_default() += 1;
        drop(inner);

        Ok(())
    }

    /// The unexpired occupants of a namespace, still individually signed.
    pub fn occupants(&self, ns: NamespaceId) -> Vec<Signed<Presence>> {
        let now = now();
        self.0
            .lock()
            .spaces
            .get(&ns)
            .map_or_else(Vec::new, |occupants| {
                occupants
                    .iter()
                    .filter(|o| o.expires > now)
                    .map(|o| o.signed.clone())
                    .collect()
            })
    }

    /// Spaces active within `window`, most recently active first, then
    /// busiest.
    pub fn active(&self, window: Duration) -> Vec<ActiveSpace> {
        let now = now();
        let cutoff = now - i64::try_from(window.as_secs()).unwrap_or(i64::MAX);

        let mut out = self
            .0
            .lock()
            .spaces
            .iter()
            .filter_map(|(ns, occupants)| {
                let seen = occupants
                    .iter()
                    .filter(|o| o.last_seen > cutoff && o.expires > now);
                let last_seen = seen.clone().map(|o| o.last_seen).max()?;
                Some(ActiveSpace {
                    ns:        *ns,
                    occupants: seen.count(),
                    idle_secs: u64::try_from(now - last_seen).unwrap_or(0),
                })
            })
            .collect::<Vec<_>>();

        // By minute, as the view publishes it, so heartbeats arriving in a
        // different order do not reshuffle the ranks.
        out.sort_by_key(|s| {
            (
                s.idle_secs / 60,
                std::cmp::Reverse(s.occupants),
                *s.ns.as_bytes(),
            )
        });
        out
    }

    /// Drops every presence that expired or went unrefreshed for `window`.
    pub fn sweep(&self, window: Duration) {
        let now = now();
        let cutoff = now - i64::try_from(window.as_secs()).unwrap_or(i64::MAX);

        let mut inner = self.0.lock();
        let spaces = inner.spaces.keys().copied().collect::<Vec<_>>();
        for ns in spaces {
            inner.remove(ns, |o| o.last_seen <= cutoff || o.expires <= now);
        }
    }
}

impl Inner {
    /// Removes `ns`'s occupants matching `stale`, keeping `by_did` in step.
    fn remove(&mut self, ns: NamespaceId, stale: impl Fn(&Occupant) -> bool) {
        let Some(occupants) = self.spaces.get_mut(&ns) else {
            return;
        };

        let mut gone = Vec::new();
        occupants.retain(|o| {
            let keep = !stale(o);
            if !keep {
                gone.push(o.did.clone());
            }
            keep
        });
        if occupants.is_empty() {
            self.spaces.remove(&ns);
        }

        for did in gone {
            let Some(held) = self.by_did.get_mut(&did) else {
                continue;
            };
            if let Some(count) = held.get_mut(&ns) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    held.remove(&ns);
                }
            }
            if held.is_empty() {
                self.by_did.remove(&did);
            }
        }
    }

    /// Drops `did`'s expired presences, so a lapsed one does not count against
    /// its caps.
    fn expire_did(&mut self, did: &Did, now: i64) {
        let spaces = self
            .by_did
            .get(did)
            .map_or_else(Vec::new, |held| held.keys().copied().collect::<Vec<_>>());
        for ns in spaces {
            self.remove(ns, |o| &o.did == did && o.expires <= now);
        }
    }
}

fn now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use iroh::SecretKey;
    use unavi_identity::signed::Signable;
    use xdid::method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    };

    use super::*;

    fn presence(key: &P256KeyPair, ns: u8, endpoint: EndpointId) -> (Presence, Signed<Presence>) {
        let presence = Presence {
            did: key.public().to_did(),
            endpoint,
            ns: NamespaceId::from(&[ns; 32]),
            expires: now() + 60,
        };
        let signed = presence.sign(key).expect("sign");
        (presence, signed)
    }

    #[test]
    fn one_did_cannot_fill_every_namespace() {
        let (table, config) = (PresenceTable::default(), Config::default());
        let (key, endpoint) = (P256KeyPair::generate(), SecretKey::generate().public());

        for ns in 0..config.max_presences_per_did {
            let (p, s) = presence(&key, u8::try_from(ns).expect("small"), endpoint);
            table.insert(&p, s, &config).expect("within the cap");
        }

        let (p, s) = presence(&key, 200, endpoint);
        assert_eq!(
            table.insert(&p, s, &config),
            Err(RegistryError::TooManyPresences)
        );
    }

    #[test]
    fn a_refresh_replaces_rather_than_adds() {
        let (table, config) = (PresenceTable::default(), Config::default());
        let (key, endpoint) = (P256KeyPair::generate(), SecretKey::generate().public());

        for _ in 0..3 {
            let (p, s) = presence(&key, 1, endpoint);
            table.insert(&p, s, &config).expect("refresh");
        }

        assert_eq!(table.occupants(NamespaceId::from(&[1; 32])).len(), 1);
        let did = Did::from_str(&key.public().to_did().to_string()).expect("did");
        assert_eq!(table.0.lock().by_did[&did][&NamespaceId::from(&[1; 32])], 1);
    }
}
