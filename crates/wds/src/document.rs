use std::{
    ops::Deref,
    sync::Arc,
};

use bytes::Bytes;
use iroh_blobs::{
    Hash,
    api::blobs::Blobs,
};
use iroh_docs::{
    AuthorId,
    Entry,
    api::Doc,
    store::Query,
};
use n0_future::StreamExt;

/// An open document, with a fixed author for writes.
#[derive(Clone, Debug)]
pub struct Document {
    handle: Arc<DocHandle>,
    blobs:  Blobs,
    author: AuthorId,
}

/// Holds a replica open; closes it when dropped.
#[derive(Debug)]
struct DocHandle {
    doc:     Doc,
    /// The runtime the document was opened on. A [`Document`] may be dropped
    /// outside an async context, so the close is spawned here.
    #[cfg(not(target_family = "wasm"))]
    runtime: tokio::runtime::Handle,
}

impl Drop for DocHandle {
    fn drop(&mut self) {
        let doc = self.doc.clone();

        let close = async move {
            if let Err(err) = doc.close().await {
                tracing::debug!(?err, "failed to close document");
            }
        };

        cfg_select! {
            target_family = "wasm" => drop(n0_future::task::spawn(close)),
            _ => drop(self.runtime.spawn(close)),
        };
    }
}

impl Deref for Document {
    type Target = Doc;

    fn deref(&self) -> &Doc {
        &self.handle.doc
    }
}

impl Document {
    pub(crate) fn new(doc: Doc, blobs: Blobs, author: AuthorId) -> anyhow::Result<Self> {
        let handle = DocHandle {
            doc,
            #[cfg(not(target_family = "wasm"))]
            runtime: tokio::runtime::Handle::try_current()?,
        };

        Ok(Self {
            handle: Arc::new(handle),
            blobs,
            author,
        })
    }

    /// The content at `key`, or `None` if the document holds no entry there or
    /// its value has not been downloaded yet.
    pub async fn get(&self, key: &str) -> anyhow::Result<Option<Bytes>> {
        let query = Query::single_latest_per_key().key_exact(key);
        let Some(entry) = self.get_one(query).await? else {
            return Ok(None);
        };
        self.value(&entry).await
    }

    /// The latest entry per key under each prefix.
    pub async fn list(&self, prefixes: &[&str]) -> anyhow::Result<Vec<Entry>> {
        let mut out = Vec::new();
        for prefix in prefixes {
            let query = Query::single_latest_per_key().key_prefix(*prefix);
            let mut stream = Box::pin(self.get_many(query).await?);
            while let Some(entry) = stream.next().await {
                out.push(entry?);
            }
        }
        Ok(out)
    }

    /// An entry's content, or `None` if it has not been downloaded yet.
    pub async fn value(&self, entry: &Entry) -> anyhow::Result<Option<Bytes>> {
        let hash = entry.content_hash();
        if !self.blobs.has(hash).await? {
            return Ok(None);
        }
        Ok(Some(self.blobs.get_bytes(hash).await?))
    }

    /// How many bytes this document's entries claim, downloaded or not.
    ///
    /// Two keys holding identical bytes count twice while the blob store keeps
    /// one copy, so this is an upper bound on what the document costs.
    pub async fn size(&self) -> anyhow::Result<u64> {
        let mut stream = Box::pin(self.get_many(Query::single_latest_per_key()).await?);
        let mut total = 0;
        while let Some(entry) = stream.next().await {
            total += entry?.content_len();
        }
        Ok(total)
    }

    pub async fn set(
        &self,
        key: impl Into<Bytes>,
        value: impl Into<Bytes>,
    ) -> anyhow::Result<Hash> {
        self.set_bytes(self.author, key, value).await
    }

    /// Points `key` at content already addressed by `hash`, uploading nothing.
    ///
    /// `size` is the content's recorded length, which for content not
    /// downloaded is only what its provider claimed.
    pub async fn set_hash(
        &self,
        key: impl Into<Bytes>,
        hash: Hash,
        size: u64,
    ) -> anyhow::Result<()> {
        self.handle.doc.set_hash(self.author, key, hash, size).await
    }

    /// Removes every entry under `prefix` this store authored, returning how
    /// many were removed.
    ///
    /// Entries other peers authored are left alone; removing one of those means
    /// writing an empty value, which wins by timestamp and reads as absence.
    pub async fn remove(&self, prefix: impl Into<Bytes>) -> anyhow::Result<usize> {
        self.del(self.author, prefix).await
    }

    /// Enrols in the sync set, so incoming requests for this namespace are
    /// answered.
    ///
    /// A namespace outside the sync set rejects every request with `NotFound`.
    /// The empty peer list enrols without dialing anyone, and the sync engine
    /// holds its own handle, so enrolment outlives the document that asked.
    pub async fn serve(&self) -> anyhow::Result<()> {
        self.start_sync(Vec::new()).await?;
        Ok(())
    }
}