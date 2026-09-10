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

/// An open document, with a set author for writes.
#[derive(Clone, Debug)]
pub struct Document {
    handle: Arc<DocHandle>,
    blobs:  Blobs,
    author: AuthorId,
}

/// Holds a replica open, closes when dropped.
#[derive(Debug)]
struct DocHandle {
    doc:     Doc,
    /// Runtime the document was opened on. Used during [`Drop::drop`] to spawn
    /// a task, allowing documents to be dropped outside of async contexts.
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

    /// The latest entry at `key`.
    async fn entry(&self, key: &str) -> anyhow::Result<Option<Entry>> {
        let query = Query::single_latest_per_key().key_exact(key);
        self.get_one(query).await
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
    pub async fn value(&self, entry: &Entry) -> Option<Bytes> {
        self.blobs.get_bytes(entry.content_hash()).await.ok()
    }

    /// The content at `key`, or `None` if the document holds no entry there or
    /// its value has not been downloaded yet.
    pub async fn get(&self, key: &str) -> anyhow::Result<Option<Bytes>> {
        let Some(entry) = self.entry(key).await? else {
            return Ok(None);
        };
        Ok(self.value(&entry).await)
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
    /// `size` is the content's length as recorded in the entry, which for
    /// content not downloaded yet is only what its provider claimed.
    pub async fn set_hash(
        &self,
        key: impl Into<Bytes>,
        hash: Hash,
        size: u64,
    ) -> anyhow::Result<()> {
        self.handle.doc.set_hash(self.author, key, hash, size).await
    }

    /// Removes every entry under `prefix` that this node authored, returning
    /// how many were removed.
    ///
    /// Entries other peers authored are left alone. Removing one of those means
    /// writing an empty value, which wins by timestamp and reads as absence on
    /// every peer.
    pub async fn remove(&self, prefix: impl Into<Bytes>) -> anyhow::Result<usize> {
        self.del(self.author, prefix).await
    }

    /// Enrols in the sync set, so incoming requests for this namespace are
    /// answered.
    ///
    /// A namespace outside the sync set rejects every incoming request with
    /// `NotFound`. The empty peer list enrols without dialing anyone.
    ///
    /// The sync engine takes a handle of its own, so enrolment outlives the
    /// document that asked for it.
    pub async fn serve(&self) -> anyhow::Result<()> {
        self.start_sync(Vec::new()).await?;
        Ok(())
    }

    /// How many bytes of content this document's entries claim, whether or not
    /// they have been downloaded.
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
}
