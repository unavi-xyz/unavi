//! Live occupancy, held in memory, bounded per DID and per namespace, and
//! expired by clock.

use std::{
    collections::HashMap,
    time::Duration,
};

use iroh::EndpointId;
use iroh_docs::NamespaceId;
use parking_lot::Mutex;
use smol_str::SmolStr;
use time::OffsetDateTime;
use unavi_identity::signed::Signed;
use xdid::core::did::Did;

use crate::{
    claim::Presence,
    error::RegistryError,
    server::config::{
        Config,
        is_did_key,
    },
};

#[derive(Default)]
pub struct PresenceTable(Mutex<Inner>);

#[derive(Default)]
struct Inner {
    spaces:  HashMap<NamespaceId, Vec<Occupant>>,
    /// Devices each DID has present, per namespace.
    by_did:  HashMap<Did, HashMap<NamespaceId, usize>>,
    /// Occupants across every namespace.
    total:   usize,
    /// Occupants whose DID is a `did:key`.
    did_key: usize,
}

pub struct ActiveSpace {
    pub ns:        NamespaceId,
    pub occupants: usize,
    pub idle_secs: u64,
    /// The space's listing title, which is the name in its document's metadata
    /// as submitted, so a client can name it without fetching the listing.
    pub title:     SmolStr,
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
        let refreshing = occupants.is_some_and(|list| {
            list.iter()
                .any(|o| o.endpoint == presence.endpoint && o.did == presence.did)
        });

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
            if inner.total >= config.max_presences
                || (is_did_key(&presence.did) && inner.did_key >= config.max_did_key_presences)
            {
                return Err(RegistryError::TooManyPresences);
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
        inner.total += 1;
        if is_did_key(&presence.did) {
            inner.did_key += 1;
        }
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

    /// Spaces active within `window` that `title` names, most recently active
    /// first, then busiest.
    ///
    /// Presence can name any namespace, so the caller says which may be
    /// published by returning the title of each live space and `None` for the
    /// rest.
    pub fn active(
        &self,
        window: Duration,
        title: impl Fn(NamespaceId) -> Option<SmolStr>,
    ) -> Vec<ActiveSpace> {
        let now = now();
        let cutoff = now - i64::try_from(window.as_secs()).unwrap_or(i64::MAX);

        let mut out = self
            .0
            .lock()
            .spaces
            .iter()
            .filter_map(|(ns, occupants)| {
                let title = title(*ns)?;
                let seen = occupants
                    .iter()
                    .filter(|o| o.last_seen > cutoff && o.expires > now);
                let last_seen = seen.clone().map(|o| o.last_seen).max()?;
                Some(ActiveSpace {
                    ns: *ns,
                    occupants: seen.count(),
                    idle_secs: u64::try_from(now - last_seen).unwrap_or(0),
                    title,
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
            debug_assert!(self.total > 0, "presence total drifted");
            self.total = self.total.saturating_sub(1);
            if is_did_key(&did) {
                debug_assert!(self.did_key > 0, "did:key presence count drifted");
                self.did_key = self.did_key.saturating_sub(1);
            }
            let Some(held) = self.by_did.get_mut(&did) else {
                continue;
            };
            if let Some(count) = held.get_mut(&ns) {
                debug_assert!(*count > 0, "per-namespace presence count drifted");
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
        let inner = table.0.lock();
        assert_eq!(inner.by_did[&did][&NamespaceId::from(&[1; 32])], 1);
        assert_eq!((inner.total, inner.did_key), (1, 1));
    }

    #[test]
    fn an_endpoint_rebound_to_a_new_did_is_not_a_refresh() {
        let table = PresenceTable::default();
        let config = Config {
            max_did_key_presences: 1,
            ..Config::default()
        };
        let shared = endpoint();
        let (p, s) = presence(&P256KeyPair::generate(), 1, shared);
        table.insert(&p, s, &config).expect("first");

        let (p, s) = presence(&P256KeyPair::generate(), 1, shared);
        assert_eq!(
            table.insert(&p, s, &config),
            Err(RegistryError::TooManyPresences)
        );
    }
    fn web_presence(name: &str, ns: u8, endpoint: EndpointId) -> (Presence, Signed<Presence>) {
        let key = P256KeyPair::generate();
        let presence = Presence {
            did: Did::from_str(&format!("did:web:{name}.example")).expect("did"),
            endpoint,
            ns: NamespaceId::from(&[ns; 32]),
            expires: now() + 60,
        };
        let signed = presence.sign(&key).expect("sign");
        (presence, signed)
    }

    fn endpoint() -> EndpointId {
        SecretKey::generate().public()
    }

    #[test]
    fn did_keys_share_one_presence_pool_while_a_did_web_is_still_admitted() {
        let table = PresenceTable::default();
        let config = Config {
            max_did_key_presences: 2,
            ..Config::default()
        };
        for ns in 0..2 {
            let (p, s) = presence(&P256KeyPair::generate(), ns, endpoint());
            table.insert(&p, s, &config).expect("within the pool");
        }

        let (p, s) = presence(&P256KeyPair::generate(), 9, endpoint());
        assert_eq!(
            table.insert(&p, s, &config),
            Err(RegistryError::TooManyPresences)
        );
        let (p, s) = web_presence("a", 9, endpoint());
        assert_eq!(table.insert(&p, s, &config), Ok(()));
    }

    #[test]
    fn presences_are_capped_across_all_namespaces_but_a_refresh_still_lands() {
        let table = PresenceTable::default();
        let config = Config {
            max_presences: 2,
            ..Config::default()
        };
        let first = endpoint();
        let (p, s) = web_presence("a", 1, first);
        table.insert(&p, s, &config).expect("first");
        let (p, s) = web_presence("b", 2, endpoint());
        table.insert(&p, s, &config).expect("second");

        let (p, s) = web_presence("c", 3, endpoint());
        assert_eq!(
            table.insert(&p, s, &config),
            Err(RegistryError::TooManyPresences)
        );
        let (p, s) = web_presence("a", 1, first);
        assert_eq!(table.insert(&p, s, &config), Ok(()));
        assert_eq!(table.0.lock().total, 2);
    }

    #[test]
    fn sweeping_releases_the_global_counts() {
        let table = PresenceTable::default();
        let config = Config::default();
        let (p, s) = presence(&P256KeyPair::generate(), 1, endpoint());
        table.insert(&p, s, &config).expect("insert");
        assert_eq!(table.0.lock().did_key, 1);

        table.sweep(Duration::ZERO);

        let inner = table.0.lock();
        assert_eq!((inner.total, inner.did_key), (0, 0));
    }

    #[test]
    fn the_active_list_holds_only_listed_namespaces_and_carries_their_titles() {
        let table = PresenceTable::default();
        let config = Config::default();
        for ns in [1, 2] {
            let (p, s) = presence(&P256KeyPair::generate(), ns, endpoint());
            table.insert(&p, s, &config).expect("insert");
        }

        let listed = NamespaceId::from(&[1; 32]);
        let active = table.active(Duration::from_mins(5), |ns| {
            (ns == listed).then(|| SmolStr::new("Plaza"))
        });

        assert_eq!(active.len(), 1);
        assert_eq!(active[0].ns, listed);
        assert_eq!(active[0].title, "Plaza");
        assert_eq!(table.occupants(NamespaceId::from(&[2; 32])).len(), 1);
    }
}
