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
    claim::Submission,
    error::RegistryError,
    server::config::Config,
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

/// The durable record of every listing. Clients sync
/// [views](crate::server::views) instead.
pub struct Catalog {
    doc:   Document,
    /// Decoded from `doc`, which only this registry writes and only after
    /// verifying, so it is trusted on load rather than re-verified.
    index: RwLock<HashMap<NamespaceId, Submission>>,
    /// Serializes check-then-write, so two submitters racing for one
    /// namespace cannot both pass the ownership check.
    write: tokio::sync::Mutex<()>,
}

impl Catalog {
    pub async fn create(store: &Store) -> anyhow::Result<Self> {
        let doc = store.named(KEY).await?;
        let now = now();
        let mut index = HashMap::new();

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
                index.insert(submission.ns, submission);
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

        {
            let index = self.index.read();
            let live = || index.values().filter(|s| s.expires > now);

            match index.get(&submission.ns) {
                Some(held) if held.expires > now && held.did != submission.did => {
                    return Err(RegistryError::NamespaceTaken);
                }
                Some(held) if held.expires > now => {}
                _ => {
                    if live().filter(|s| s.did == submission.did).count()
                        >= config.max_submissions_per_did
                    {
                        return Err(RegistryError::TooManySubmissions);
                    }
                    if live().count() >= config.max_listings {
                        return Err(RegistryError::TooManySubmissions);
                    }
                }
            }
        }

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

        self.index.write().insert(submission.ns, submission);
        Ok(())
    }

    /// Removes `did`'s listing of `ns`. Refused unless `did` holds it.
    pub async fn remove(&self, ns: NamespaceId, did: &Did) -> Result<(), RegistryError> {
        let _write = self.write.lock().await;

        let held = self
            .index
            .read()
            .get(&ns)
            .is_some_and(|held| &held.did == did && held.expires > now());
        if !held {
            return Err(RegistryError::NotPermitted);
        }

        self.doc.remove_key(entry_key(ns)).await.map_err(|err| {
            tracing::warn!(?err, "failed removing listing");
            RegistryError::Internal
        })?;
        self.index.write().remove(&ns);
        Ok(())
    }

    /// Every unexpired listing.
    #[must_use]
    pub fn live(&self) -> Vec<Submission> {
        let now = now();
        self.index
            .read()
            .values()
            .filter(|s| s.expires > now)
            .cloned()
            .collect()
    }

    /// Deletes expired listings from the document and the index.
    pub async fn prune(&self) -> anyhow::Result<usize> {
        let _write = self.write.lock().await;
        let now = now();

        let expired = self
            .index
            .read()
            .values()
            .filter(|s| s.expires <= now)
            .map(|s| s.ns)
            .collect::<Vec<_>>();

        for ns in &expired {
            self.doc.remove_key(entry_key(*ns)).await?;
            self.index.write().remove(ns);
        }

        Ok(expired.len())
    }
}

fn now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}
