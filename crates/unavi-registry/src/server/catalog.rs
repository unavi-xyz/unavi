//! Every live listing, durable in a document only this registry writes, and
//! indexed in memory so no request reads the whole document.

use std::collections::HashMap;

use iroh_docs::NamespaceId;
use parking_lot::RwLock;
use serde::{
    Deserialize,
    Serialize,
};
use time::OffsetDateTime;
use unavi_identity::{
    authorship::Authorship,
    signed::Signed,
};
use unavi_store::{
    Document,
    Store,
};
use xdid::core::did::Did;

use crate::{
    claim::{
        Kind,
        Submission,
    },
    error::RegistryError,
    server::config::{
        Config,
        is_did_key,
    },
};

const ENTRIES_PREFIX: &str = "entries/";

/// Where the catalog namespace is recorded across restarts.
const KEY: &str = "registry/catalog";

fn entry_key(ns: NamespaceId) -> String {
    format!("{ENTRIES_PREFIX}{ns}")
}

/// A listing as stored: the submission and the authorship it was accepted on,
/// both still signed, so the record can be re-verified by anyone.
#[derive(Serialize, Deserialize)]
struct Listing {
    submission: Signed<Submission>,
    authorship: Authorship,
}

/// The listings held in memory, with the counts the caps are checked against
/// kept in step so no check scans them.
///
/// Expired entries stay counted until pruned.
#[derive(Default)]
struct Index {
    entries: HashMap<NamespaceId, Submission>,
    per_did: HashMap<Did, usize>,
    /// Entries held by `did:key` identities.
    did_key: usize,
}

impl Index {
    fn insert(&mut self, submission: Submission) {
        self.remove(submission.ns);
        *self.per_did.entry(submission.did.clone()).or_default() += 1;
        if is_did_key(&submission.did) {
            self.did_key += 1;
        }
        self.entries.insert(submission.ns, submission);
    }

    fn remove(&mut self, ns: NamespaceId) {
        let Some(gone) = self.entries.remove(&ns) else {
            return;
        };
        if let Some(count) = self.per_did.get_mut(&gone.did) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.per_did.remove(&gone.did);
            }
        }
        if is_did_key(&gone.did) {
            self.did_key = self.did_key.saturating_sub(1);
        }
    }

    /// Whether `submission` may be recorded at `now`.
    ///
    /// A listing its DID already holds is refreshed past every cap, and an
    /// expired one held by another DID is replaced like a free namespace.
    fn admit(
        &self,
        submission: &Submission,
        now: i64,
        config: &Config,
    ) -> Result<(), RegistryError> {
        match self.entries.get(&submission.ns) {
            Some(held) if held.did == submission.did => return Ok(()),
            Some(held) if held.expires > now => return Err(RegistryError::NamespaceTaken),
            _ => {}
        }

        let held = self.per_did.get(&submission.did).copied().unwrap_or(0);
        if held >= config.max_submissions_per_did
            || self.entries.len() >= config.max_listings
            || (is_did_key(&submission.did) && self.did_key >= config.max_did_key_listings)
        {
            return Err(RegistryError::TooManySubmissions);
        }
        Ok(())
    }

    fn is_live_space(&self, ns: NamespaceId, now: i64) -> bool {
        self.entries
            .get(&ns)
            .is_some_and(|s| s.kind == Kind::Space && s.expires > now)
    }
}

/// The durable record of every listing. Clients sync
/// [views](crate::server::views) instead.
pub struct Catalog {
    doc:   Document,
    /// Decoded from `doc`, which only this registry writes and only after
    /// verifying, so it is trusted on load rather than re-verified.
    index: RwLock<Index>,
    /// Serializes check-then-write, so two submitters racing for one
    /// namespace cannot both pass the ownership check.
    write: tokio::sync::Mutex<()>,
}

impl Catalog {
    pub async fn create(store: &Store) -> anyhow::Result<Self> {
        let doc = store.named(KEY).await?;
        let now = now();
        let mut index = Index::default();

        for entry in doc.list(ENTRIES_PREFIX).await? {
            let Some(bytes) = doc.value(&entry).await? else {
                continue;
            };
            let Some(submission) = postcard::from_bytes::<Listing>(&bytes)
                .ok()
                .and_then(|listing| listing.submission.payload().ok())
            else {
                continue;
            };
            if submission.expires > now {
                index.insert(submission);
            }
        }

        Ok(Self {
            doc,
            index: RwLock::new(index),
            write: tokio::sync::Mutex::new(()),
        })
    }

    /// Records a verified listing, refusing a namespace another DID holds and a
    /// caller past its share.
    ///
    /// Refreshing a listing the caller already holds is always allowed; only a
    /// new namespace counts against the caps.
    pub async fn insert(
        &self,
        submission: Submission,
        signed: Signed<Submission>,
        authorship: Authorship,
        config: &Config,
    ) -> Result<(), RegistryError> {
        let _write = self.write.lock().await;
        let now = now();

        self.index.read().admit(&submission, now, config)?;

        let listing = Listing {
            submission: signed,
            authorship,
        };
        let value = postcard::to_stdvec(&listing).map_err(|_| RegistryError::Internal)?;
        self.doc
            .set(entry_key(submission.ns), value)
            .await
            .map_err(|err| {
                tracing::warn!(?err, "failed writing listing");
                RegistryError::Internal
            })?;

        self.index.write().insert(submission);
        Ok(())
    }

    /// Removes `did`'s listing of `ns`. Refused unless `did` holds it.
    pub async fn remove(&self, ns: NamespaceId, did: &Did) -> Result<(), RegistryError> {
        let _write = self.write.lock().await;

        let held = self
            .index
            .read()
            .entries
            .get(&ns)
            .is_some_and(|held| &held.did == did && held.expires > now());
        if !held {
            return Err(RegistryError::NotPermitted);
        }

        self.doc.remove_key(entry_key(ns)).await.map_err(|err| {
            tracing::warn!(?err, "failed removing listing");
            RegistryError::Internal
        })?;
        self.index.write().remove(ns);
        Ok(())
    }

    /// Every unexpired listing.
    #[must_use]
    pub fn live(&self) -> Vec<Submission> {
        let now = now();
        self.index
            .read()
            .entries
            .values()
            .filter(|s| s.expires > now)
            .cloned()
            .collect()
    }

    /// Whether `ns` has an unexpired listing of [`Kind::Space`].
    #[must_use]
    pub fn is_live_space(&self, ns: NamespaceId) -> bool {
        self.index.read().is_live_space(ns, now())
    }

    /// Deletes expired listings from the document and the index.
    pub async fn prune(&self) -> anyhow::Result<usize> {
        let _write = self.write.lock().await;
        let now = now();

        let expired = self
            .index
            .read()
            .entries
            .values()
            .filter(|s| s.expires <= now)
            .map(|s| s.ns)
            .collect::<Vec<_>>();

        for ns in &expired {
            self.doc.remove_key(entry_key(*ns)).await?;
            self.index.write().remove(*ns);
        }

        Ok(expired.len())
    }
}

fn now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use xdid::method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    };

    use super::*;

    const NOW: i64 = 1_000;

    fn listing(did: &Did, ns: u8, kind: Kind, expires: i64) -> Submission {
        Submission {
            did: did.clone(),
            ns: NamespaceId::from(&[ns; 32]),
            kind,
            title: "t".into(),
            description: None,
            tags: Vec::new(),
            expires,
        }
    }

    fn did_key() -> Did {
        P256KeyPair::generate().public().to_did()
    }

    fn did_web() -> Did {
        Did::from_str("did:web:example.com").expect("did")
    }

    #[test]
    fn did_keys_share_one_pool_while_a_did_web_is_still_admitted() {
        let config = Config {
            max_did_key_listings: 3,
            ..Config::default()
        };
        let mut index = Index::default();
        for ns in 0..3 {
            let sub = listing(&did_key(), ns, Kind::Space, NOW + 60);
            index.admit(&sub, NOW, &config).expect("within the pool");
            index.insert(sub);
        }

        let key_sub = listing(&did_key(), 10, Kind::Space, NOW + 60);
        assert_eq!(
            index.admit(&key_sub, NOW, &config),
            Err(RegistryError::TooManySubmissions)
        );
        let web_sub = listing(&did_web(), 11, Kind::Space, NOW + 60);
        assert_eq!(index.admit(&web_sub, NOW, &config), Ok(()));
    }

    #[test]
    fn a_did_is_capped_and_the_whole_catalog_is_capped() {
        let config = Config {
            max_submissions_per_did: 2,
            max_listings: 3,
            ..Config::default()
        };
        let (web, other) = (did_web(), did_key());
        let mut index = Index::default();
        for ns in 0..2 {
            index.insert(listing(&web, ns, Kind::Space, NOW + 60));
        }

        let third = listing(&web, 5, Kind::Space, NOW + 60);
        assert_eq!(
            index.admit(&third, NOW, &config),
            Err(RegistryError::TooManySubmissions)
        );

        index.insert(listing(&other, 6, Kind::Space, NOW + 60));
        let fourth = listing(&did_key(), 7, Kind::Space, NOW + 60);
        assert_eq!(
            index.admit(&fourth, NOW, &config),
            Err(RegistryError::TooManySubmissions)
        );
    }

    #[test]
    fn refreshing_a_held_listing_ignores_every_cap() {
        let config = Config {
            max_submissions_per_did: 1,
            max_listings: 1,
            max_did_key_listings: 1,
            ..Config::default()
        };
        let key = did_key();
        let mut index = Index::default();
        index.insert(listing(&key, 1, Kind::Space, NOW + 60));

        let refresh = listing(&key, 1, Kind::Space, NOW + 120);
        assert_eq!(index.admit(&refresh, NOW, &config), Ok(()));
        index.insert(refresh);
        assert_eq!(index.entries.len(), 1);
        assert_eq!(index.did_key, 1);
    }

    #[test]
    fn an_expired_listing_of_another_did_is_replaced_and_recounted() {
        let config = Config::default();
        let (old, new) = (did_web(), did_key());
        let mut index = Index::default();
        index.insert(listing(&old, 1, Kind::Space, NOW - 1));

        let take = listing(&new, 1, Kind::Space, NOW + 60);
        assert_eq!(index.admit(&take, NOW, &config), Ok(()));
        index.insert(take);

        assert!(!index.per_did.contains_key(&old));
        assert_eq!(index.per_did[&new], 1);
        assert_eq!(index.did_key, 1);
    }

    #[test]
    fn a_live_listing_of_another_did_is_not_replaced() {
        let mut index = Index::default();
        index.insert(listing(&did_web(), 1, Kind::Space, NOW + 60));

        let take = listing(&did_key(), 1, Kind::Space, NOW + 60);
        assert_eq!(
            index.admit(&take, NOW, &Config::default()),
            Err(RegistryError::NamespaceTaken)
        );
    }

    #[test]
    fn removing_a_listing_releases_its_counts() {
        let key = did_key();
        let mut index = Index::default();
        index.insert(listing(&key, 1, Kind::Space, NOW + 60));
        index.remove(NamespaceId::from(&[1; 32]));

        assert!(index.entries.is_empty());
        assert!(index.per_did.is_empty());
        assert_eq!(index.did_key, 0);
    }

    #[test]
    fn only_a_live_space_counts_as_a_listed_space() {
        let web = did_web();
        let mut index = Index::default();
        index.insert(listing(&web, 1, Kind::Space, NOW + 60));
        index.insert(listing(&web, 2, Kind::Avatar, NOW + 60));
        index.insert(listing(&web, 3, Kind::Space, NOW - 1));

        let is_live = |ns: u8| index.is_live_space(NamespaceId::from(&[ns; 32]), NOW);
        assert!(is_live(1));
        assert!(!is_live(2), "an avatar is not a space");
        assert!(!is_live(3), "an expired listing is not live");
        assert!(!is_live(4), "an unlisted namespace is not listed");
    }
}
