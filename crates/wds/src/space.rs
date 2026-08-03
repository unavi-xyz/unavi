//! Space-doc persistence primitive.
//!
//! A space (or nested subdocument) is an iroh-docs namespace whose `snapshot`
//! key points at a Loro snapshot blob, with one `deps/<hash>` entry per
//! transitive blob dependency so hosts pin and meter them without interpreting
//! content. The Loro encode/decode itself is the caller's concern; this module
//! deals only in bytes, hashes and namespaces.

use std::time::Duration;

use bytes::Bytes;
use iroh::EndpointAddr;
use iroh_blobs::{
    Hash,
    api::blobs::Blobs,
};
use iroh_docs::{
    AuthorId,
    Capability,
    NamespaceId,
    api::Doc,
    protocol::Docs,
    store::Query,
};

use crate::format::keys;

/// Creates a new namespace holding a Loro `snapshot` blob plus `deps/` entries
/// for its transitive blob dependencies, returning the namespace id.
pub async fn create_snapshot_doc(
    docs: &Docs,
    blobs: &Blobs,
    snapshot: Bytes,
    deps: &[(Hash, u64)],
) -> anyhow::Result<NamespaceId> {
    let author = docs.api().author_default().await?;
    let doc = docs.api().create().await?;
    let ns = doc.id();
    write_snapshot(&doc, blobs, author, snapshot, deps).await?;
    Ok(ns)
}

/// Writes (or overwrites) the `snapshot` entry and `deps/` entries of a doc.
pub async fn write_snapshot(
    doc: &Doc,
    blobs: &Blobs,
    author: AuthorId,
    snapshot: Bytes,
    deps: &[(Hash, u64)],
) -> anyhow::Result<()> {
    let len = snapshot.len() as u64;
    let info = blobs.add_bytes(snapshot).await?;
    doc.set_hash(author, keys::SNAPSHOT, info.hash, len).await?;
    for (hash, size) in deps {
        doc.set_hash(author, keys::dep(*hash), *hash, *size).await?;
    }
    Ok(())
}

/// Imports (if needed) and syncs a namespace from `peers`, then polls for its
/// `snapshot` blob up to `attempts` times, returning the bytes once available.
pub async fn fetch_snapshot(
    docs: &Docs,
    blobs: &Blobs,
    ns: NamespaceId,
    peers: Vec<EndpointAddr>,
    attempts: usize,
    delay: Duration,
) -> anyhow::Result<Option<Bytes>> {
    let doc = match docs.api().open(ns).await? {
        Some(doc) => doc,
        None => docs.api().import_namespace(Capability::Read(ns)).await?,
    };
    if !peers.is_empty() {
        let _ = doc.start_sync(peers).await;
    }

    for _ in 0..attempts.max(1) {
        let query = Query::single_latest_per_key().key_exact(keys::SNAPSHOT);
        if let Some(entry) = doc.get_one(query).await?
            && let Ok(bytes) = blobs.get_bytes(entry.content_hash()).await
        {
            return Ok(Some(bytes));
        }
        n0_future::time::sleep(delay).await;
    }
    Ok(None)
}

/// Reads the latest `snapshot` blob of a namespace, if the namespace is known
/// and its content is available locally.
pub async fn read_snapshot(
    docs: &Docs,
    blobs: &Blobs,
    ns: NamespaceId,
) -> anyhow::Result<Option<Bytes>> {
    let Some(doc) = docs.api().open(ns).await? else {
        return Ok(None);
    };
    let query = Query::single_latest_per_key().key_exact(keys::SNAPSHOT);
    let Some(entry) = doc.get_one(query).await? else {
        return Ok(None);
    };
    let bytes = blobs.get_bytes(entry.content_hash()).await?;
    Ok(Some(bytes))
}
