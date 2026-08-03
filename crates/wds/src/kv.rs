//! Generic key/value access over docs (iroh-docs namespaces).
//!
//! A doc is a map of UTF-8 `key -> value` entries; values are stored as blobs
//! and fetched on read. Writes require the doc's write capability (held for
//! docs this node created); reads only require the namespace.

use bytes::Bytes;
use futures::StreamExt;
use iroh_blobs::api::blobs::Blobs;
use iroh_docs::{
    NamespaceId,
    protocol::Docs,
    store::Query,
};

/// Creates a new writable doc, returning its namespace.
pub async fn create(docs: &Docs) -> anyhow::Result<NamespaceId> {
    Ok(docs.api().create().await?.id())
}

/// Writes a `key -> value` entry into a doc held with write capability.
pub async fn set(docs: &Docs, ns: NamespaceId, key: &str, value: Bytes) -> anyhow::Result<()> {
    let author = docs.api().author_default().await?;
    let doc = docs
        .api()
        .open(ns)
        .await?
        .ok_or_else(|| anyhow::anyhow!("doc not open"))?;
    doc.set_bytes(author, key.to_string(), value).await?;
    Ok(())
}

/// Removes entries at (or under) `key` from a doc held with write capability.
pub async fn delete(docs: &Docs, ns: NamespaceId, key: &str) -> anyhow::Result<()> {
    let author = docs.api().author_default().await?;
    let doc = docs
        .api()
        .open(ns)
        .await?
        .ok_or_else(|| anyhow::anyhow!("doc not open"))?;
    doc.del(author, key.to_string()).await?;
    Ok(())
}

/// Reads the latest value at `key`, if present.
pub async fn get(
    docs: &Docs,
    blobs: &Blobs,
    ns: NamespaceId,
    key: &str,
) -> anyhow::Result<Option<Bytes>> {
    let Some(doc) = docs.api().open(ns).await? else {
        return Ok(None);
    };
    let query = Query::single_latest_per_key().key_exact(key);
    let Some(entry) = doc.get_one(query).await? else {
        return Ok(None);
    };
    Ok(Some(blobs.get_bytes(entry.content_hash()).await?))
}

/// Lists the latest `(key, value)` entries whose key starts with `prefix`.
pub async fn list(
    docs: &Docs,
    blobs: &Blobs,
    ns: NamespaceId,
    prefix: &str,
) -> anyhow::Result<Vec<(String, Bytes)>> {
    let Some(doc) = docs.api().open(ns).await? else {
        return Ok(Vec::new());
    };
    let query = Query::single_latest_per_key().key_prefix(prefix);
    let entries = doc.get_many(query).await?;
    let mut entries = std::pin::pin!(entries);

    let mut out = Vec::new();
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let key = String::from_utf8_lossy(entry.key()).into_owned();
        if let Ok(value) = blobs.get_bytes(entry.content_hash()).await {
            out.push((key, value));
        }
    }
    Ok(out)
}
