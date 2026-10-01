use iroh_docs::NamespaceId;
use time::OffsetDateTime;
use unavi_identity::signed_bytes::SignedBytes;
use unavi_store::{
    Store,
    document::Document,
};
use xdid::resolver::DidResolver;

use crate::entry::Submission;

const ENTRIES_PREFIX: &str = "entries/";

/// Where the catalog namespace is recorded across restarts.
const KEY: &str = "registry/catalog";

fn entry_key(ns: NamespaceId) -> String {
    format!("{ENTRIES_PREFIX}{ns}")
}

/// Durable record of every live submission, written only by this registry.
/// Clients sync [views](crate::views) instead, not this doc.
pub struct Catalog {
    doc: Document,
}

impl Catalog {
    pub async fn create(store: &Store) -> anyhow::Result<Self> {
        Ok(Self {
            doc: store.open_named_doc(KEY).await?,
        })
    }

    pub async fn insert(
        &self,
        submission: &Submission,
        signed: &SignedBytes<Submission>,
    ) -> anyhow::Result<()> {
        let value = postcard::to_stdvec(signed)?;
        self.doc.set(entry_key(submission.ns), value).await?;
        Ok(())
    }

    pub async fn remove(&self, ns: NamespaceId) -> anyhow::Result<()> {
        self.doc.remove(entry_key(ns)).await?;
        Ok(())
    }

    /// Every unexpired submission whose signature still verifies.
    ///
    /// Verification is repeated on read rather than trusted from write time.
    pub async fn live(&self, resolver: &DidResolver) -> anyhow::Result<Vec<Submission>> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let mut out = Vec::new();

        for entry in self.doc.list(&[ENTRIES_PREFIX]).await? {
            let Some(bytes) = self.doc.value(&entry).await? else {
                continue;
            };
            let Ok(signed) = postcard::from_bytes::<SignedBytes<Submission>>(&bytes) else {
                continue;
            };
            let Ok(submission) = signed.payload() else {
                continue;
            };
            if submission.expires <= now {
                continue;
            }
            if signed.verify(&submission.did, resolver).await.is_err() {
                continue;
            }
            out.push(submission);
        }

        Ok(out)
    }
}
