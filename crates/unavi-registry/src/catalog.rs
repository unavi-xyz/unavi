use futures::StreamExt;
use iroh_blobs::api::blobs::Blobs;
use iroh_docs::{
    NamespaceId,
    api::Doc,
    protocol::Docs,
    store::Query,
};
use time::OffsetDateTime;
use unavi_identity::signed_bytes::SignedBytes;
use unavi_store::store::Store;
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
    ns: NamespaceId,
}

impl Catalog {
    pub async fn create(store: &Store) -> anyhow::Result<Self> {
        let ns = store.open_or_mint(KEY).await?.id();
        Ok(Self { ns })
    }

    async fn doc(&self, docs: &Docs) -> anyhow::Result<Doc> {
        docs.api()
            .open(self.ns)
            .await?
            .ok_or_else(|| anyhow::anyhow!("catalog doc {} not open", self.ns))
    }

    pub async fn insert(
        &self,
        docs: &Docs,
        submission: &Submission,
        signed: &SignedBytes<Submission>,
    ) -> anyhow::Result<()> {
        let doc = self.doc(docs).await?;
        let author = docs.api().author_default().await?;
        let value = postcard::to_stdvec(signed)?;
        doc.set_bytes(author, entry_key(submission.ns), value)
            .await?;
        Ok(())
    }

    pub async fn remove(&self, docs: &Docs, ns: NamespaceId) -> anyhow::Result<()> {
        let doc = self.doc(docs).await?;
        let author = docs.api().author_default().await?;
        doc.del(author, entry_key(ns)).await?;
        Ok(())
    }

    /// Every unexpired submission whose signature still verifies.
    ///
    /// Verification is repeated on read rather than trusted from write time.
    pub async fn live(
        &self,
        docs: &Docs,
        blobs: &Blobs,
        resolver: &DidResolver,
    ) -> anyhow::Result<Vec<Submission>> {
        let doc = self.doc(docs).await?;
        let query = Query::single_latest_per_key().key_prefix(ENTRIES_PREFIX);
        let entries = doc.get_many(query).await?;
        let mut entries = std::pin::pin!(entries);

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let mut out = Vec::new();

        while let Some(entry) = entries.next().await {
            let Ok(bytes) = blobs.get_bytes(entry?.content_hash()).await else {
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
