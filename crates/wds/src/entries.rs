//! Entry-level document access.
//!
//! Every key is its own entry rather than a whole document under one key, so
//! iroh-docs' per-key last-writer-wins merge resolves concurrent writes
//! between peers — a whole-document snapshot under one key would instead
//! have one peer's checkpoint overwrite the other's scene outright.
//!
//! Nothing here interprets a payload, and nothing tracks dependencies: an
//! entry's value *is* a blob hash, so a replicating host fetches, tags and
//! meters every byte without reading any of it. There is no `deps/`-style
//! dependency-tracking convention.

use std::time::Duration;

use bytes::Bytes;
use iroh::EndpointAddr;
use iroh_blobs::{
    Hash,
    api::blobs::Blobs,
};
use iroh_docs::{
    AuthorId,
    NamespaceId,
    api::Doc,
    engine::LiveEvent,
    protocol::Docs,
    store::Query,
};
use n0_future::{
    Stream,
    StreamExt,
};

/// One entry as stored: the value is always a hash, whether or not its content
/// has been downloaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocEntry {
    pub key:       String,
    pub hash:      Hash,
    pub size:      u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum Write {
    /// Adds the bytes to the blob store, then points the key at them.
    Bytes { key: String, value: Bytes },
    /// Points the key at content already in the blob store.
    Hash { key: String, hash: Hash, size: u64 },
    /// Writes an empty value at the key.
    ///
    /// This is the only removal that crosses authors: `del` sweeps the
    /// caller's own entries and nobody else's, but the empty entry it leaves
    /// wins by timestamp and is filtered by `get_many`, so every peer reads
    /// absence. Passing a bare prefix instead of a full key is the same
    /// operation, and is how one write sweeps a whole prim.
    Remove { key: String },
}

#[derive(Debug, Clone)]
pub enum DocChange {
    Entry(DocEntry),
    /// An entry's content finished downloading, so a value that read as
    /// unavailable is now readable.
    ContentReady(Hash),
    SyncFinished,
}

/// Creates a namespace, returning its id. The document starts empty; the
/// caller writes `meta/` and its prims.
pub async fn create(docs: &Docs) -> anyhow::Result<NamespaceId> {
    Ok(docs.api().create().await?.id())
}

pub async fn author(docs: &Docs) -> anyhow::Result<AuthorId> {
    docs.api().author_default().await
}

/// Lists the latest entry per key under each prefix.
///
/// Empty entries are filtered by `get_many`, so a cross-author tombstone reads
/// as absence here exactly as it does on every other peer.
pub async fn list(doc: &Doc, prefixes: &[&str]) -> anyhow::Result<Vec<DocEntry>> {
    let mut out = Vec::new();
    for prefix in prefixes {
        let query = Query::single_latest_per_key().key_prefix(*prefix);
        let mut stream = Box::pin(doc.get_many(query).await?);
        while let Some(entry) = stream.next().await {
            let entry = entry?;
            out.push(DocEntry {
                key:       String::from_utf8_lossy(entry.key()).into_owned(),
                hash:      entry.content_hash(),
                size:      entry.content_len(),
                timestamp: entry.timestamp(),
            });
        }
    }
    Ok(out)
}

/// Reads an entry's content, or `None` if it has not been downloaded yet.
pub async fn value(blobs: &Blobs, entry: &DocEntry) -> Option<Bytes> {
    blobs.get_bytes(entry.hash).await.ok()
}

/// Applies writes in order. Order is load-bearing for deletion: a `Sweep` at
/// `p/<prim>/` would eat a tombstone written at `p/<prim>/parent/` before it.
pub async fn apply(
    doc: &Doc,
    blobs: &Blobs,
    author: AuthorId,
    writes: impl IntoIterator<Item = Write>,
) -> anyhow::Result<()> {
    for write in writes {
        match write {
            Write::Bytes { key, value } => {
                let size = value.len() as u64;
                let info = blobs.add_bytes(value).await?;
                doc.set_hash(author, key, info.hash, size).await?;
            }
            Write::Hash { key, hash, size } => {
                doc.set_hash(author, key, hash, size).await?;
            }
            Write::Remove { key } => {
                doc.del(author, key).await?;
            }
        }
    }
    Ok(())
}

/// Streams live edits. Entries arrive unordered — a child may be seen before
/// its parent — which is why the consumer holds orphans rather than reparenting
/// them.
pub async fn subscribe(
    doc: &Doc,
) -> anyhow::Result<impl Stream<Item = DocChange> + Send + Unpin + 'static> {
    let stream = doc.subscribe().await?;
    Ok(Box::pin(stream.filter_map(|event| match event {
        Ok(LiveEvent::InsertLocal { entry } | LiveEvent::InsertRemote { entry, .. }) => {
            Some(DocChange::Entry(DocEntry {
                key:       String::from_utf8_lossy(entry.key()).into_owned(),
                hash:      entry.content_hash(),
                size:      entry.content_len(),
                timestamp: entry.timestamp(),
            }))
        }
        Ok(LiveEvent::ContentReady { hash }) => Some(DocChange::ContentReady(hash)),
        Ok(LiveEvent::SyncFinished(_)) => Some(DocChange::SyncFinished),
        _ => None,
    })))
}

/// Imports and syncs a namespace from `peers`, then polls until it holds
/// entries under `prefix`.
pub async fn fetch(
    docs: &Docs,
    ns: NamespaceId,
    peers: Vec<EndpointAddr>,
    prefix: &str,
    attempts: usize,
    delay: Duration,
) -> anyhow::Result<Option<Doc>> {
    let doc = crate::docs::ensure_open(docs, ns).await?;
    if !peers.is_empty() {
        let _ = doc.start_sync(peers).await;
    }

    for _ in 0..attempts.max(1) {
        let query = Query::single_latest_per_key().key_prefix(prefix);
        if doc.get_one(query).await?.is_some() {
            return Ok(Some(doc));
        }
        n0_future::time::sleep(delay).await;
    }
    Ok(None)
}
