//! Every key is its own entry, so iroh-docs' per-key last-writer-wins merge
//! resolves concurrent writes between peers. An entry's value is always a blob
//! hash, so a replicating host fetches, tags and meters every byte without
//! reading any of it; no dependency-tracking convention exists on top.

use std::time::Duration;

use bytes::Bytes;
use iroh::EndpointAddr;
use iroh_blobs::{
    Hash,
    api::blobs::Blobs,
};
use iroh_docs::{
    AuthorId,
    Entry,
    NamespaceId,
    api::Doc,
    store::Query,
};
use n0_future::StreamExt;

/// One document open on this node, with the blob store its values live in and
/// the author this node writes under.
#[derive(Clone, Debug)]
pub struct Namespace {
    doc:    Doc,
    blobs:  Blobs,
    author: AuthorId,
}

impl Namespace {
    pub(crate) const fn new(doc: Doc, blobs: Blobs, author: AuthorId) -> Self {
        Self { doc, blobs, author }
    }

    #[must_use]
    pub fn id(&self) -> NamespaceId {
        self.doc.id()
    }

    /// The latest entry at exactly `key`, or `None` if the document holds none.
    ///
    /// An empty entry is filtered by `get_one`, so a tombstone reads as absence
    /// here exactly as it does on every other peer.
    pub async fn get(&self, key: &str) -> anyhow::Result<Option<Entry>> {
        let query = Query::single_latest_per_key().key_exact(key);
        self.doc.get_one(query).await
    }

    /// The latest entry per key under each prefix.
    ///
    /// Empty entries are filtered by `get_many`, so a cross-author tombstone
    /// reads as absence here exactly as it does on every other peer.
    pub async fn list(&self, prefixes: &[&str]) -> anyhow::Result<Vec<Entry>> {
        let mut out = Vec::new();
        for prefix in prefixes {
            let query = Query::single_latest_per_key().key_prefix(*prefix);
            let mut stream = Box::pin(self.doc.get_many(query).await?);
            while let Some(entry) = stream.next().await {
                out.push(entry?);
            }
        }
        Ok(out)
    }

    /// An entry's content, or `None` if it has not been downloaded yet.
    pub async fn value(&self, entry: &Entry) -> Option<Bytes> {
        self.blobs.get_bytes(entry.content_hash()).await.ok()
    }

    pub async fn set(
        &self,
        key: impl Into<Bytes>,
        value: impl Into<Bytes>,
    ) -> anyhow::Result<Hash> {
        self.doc.set_bytes(self.author, key, value).await
    }

    /// Removes every entry under `prefix` that this node authored, returning
    /// how many went.
    ///
    /// Entries other peers authored are untouched: removing one of those has to
    /// be written as an empty value, which wins by timestamp and reads as
    /// absence everywhere.
    pub async fn remove(&self, prefix: impl Into<Bytes>) -> anyhow::Result<usize> {
        self.doc.del(self.author, prefix).await
    }

    /// Enrols in the sync set, so incoming requests for this namespace are
    /// answered.
    ///
    /// A namespace absent from the sync set rejects every incoming request with
    /// `NotFound`; an empty peer list enrols without dialing anyone.
    pub async fn serve(&self) -> anyhow::Result<()> {
        self.doc.start_sync(Vec::new()).await?;
        Ok(())
    }

    /// Enrols in the sync set and reconciles with `peers`.
    pub async fn sync_from(&self, peers: Vec<EndpointAddr>) -> anyhow::Result<()> {
        self.doc.start_sync(peers).await?;
        Ok(())
    }

    /// Polls until an entry exists under `prefix`, reporting whether one
    /// arrived before the attempts ran out.
    pub async fn wait_for(
        &self,
        prefix: &str,
        attempts: usize,
        delay: Duration,
    ) -> anyhow::Result<bool> {
        for _ in 0..attempts.max(1) {
            let query = Query::single_latest_per_key().key_prefix(prefix);
            if self.doc.get_one(query).await?.is_some() {
                return Ok(true);
            }
            n0_future::time::sleep(delay).await;
        }
        Ok(false)
    }
}
