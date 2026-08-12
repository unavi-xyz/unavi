use futures::StreamExt;
use iroh_blobs::api::blobs::Blobs;
use iroh_docs::{
    NamespaceId,
    api::Doc,
    protocol::Docs,
    store::Query,
};
use time::OffsetDateTime;
use wds::signed_bytes::SignedBytes;

use crate::entry::Submission;

const ENTRIES_PREFIX: &str = "entries/";

fn entry_key(ns: NamespaceId) -> String {
    format!("{ENTRIES_PREFIX}{ns}")
}

/// Durable record of every live submission, written only by this registry.
/// Clients sync [views](crate::views) instead, not this doc.
pub struct Catalog {
    ns: NamespaceId,
}

impl Catalog {
    pub async fn create(docs: &Docs) -> anyhow::Result<Self> {
        let doc = docs.api().create().await?;
        Ok(Self { ns: doc.id() })
    }

    #[must_use]
    pub const fn namespace(&self) -> NamespaceId {
        self.ns
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
    pub async fn live(&self, docs: &Docs, blobs: &Blobs) -> anyhow::Result<Vec<Submission>> {
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
            if !wds::signed_bytes::verify_did_signature(&signed, &submission.did).await {
                continue;
            }
            out.push(submission);
        }

        Ok(out)
    }
}
