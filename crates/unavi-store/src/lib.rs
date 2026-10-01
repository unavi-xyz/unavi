//! Document and blob storage, backed by `iroh-docs` and `iroh-blobs`.
//!
//! A [`Document`] is a key-value map, identified by a "namespace" ID. Each key
//! is a separately signed entry, so peers writing different keys merge
//! and peers writing the same key resolve by timestamp. An entry stores
//! the hash of its value, so values repeated across keys occupy only one copy
//! on disk, within the blob store.

use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use iroh::protocol::RouterBuilder;
use iroh_blobs::api::{
    Store as BlobStore,
    blobs::Blobs,
};
use iroh_docs::{
    AuthorId,
    Capability,
    CapabilityKind,
    NamespaceId,
    api::Doc,
    protocol::Docs,
};
use iroh_gossip::net::Gossip;
use n0_future::{
    StreamExt,
    task::AbortOnDropHandle,
    time::SystemTime,
};
use unavi_local::LocalStorage;

use crate::document::Document;

pub mod builder;
pub mod document;
mod retention;

/// Placeholder non-empty value for marking a document as visited.
const VISITED: &[u8] = b"1";

/// A released [`Document`] closes on a spawned task, so a replica nothing uses
/// can still read as held for a moment.
const REMOVE_ATTEMPTS: usize = 10;
const REMOVE_DELAY: Duration = Duration::from_millis(20);

/// The concrete blob store, chosen by target and configured storage.
trait OwnedBlobs: AsRef<BlobStore> + std::fmt::Debug + Send + Sync {}
impl<T: AsRef<BlobStore> + std::fmt::Debug + Send + Sync> OwnedBlobs for T {}
type BoxedBlobs = Box<dyn OwnedBlobs>;

/// A cloneable handle to the store.
#[derive(Clone, Debug)]
pub struct Store(Arc<StoreInner>);

#[derive(Debug)]
struct StoreInner {
    blobs:      BoxedBlobs,
    docs:       Docs,
    gossip:     Gossip,
    author:     AuthorId,
    storage:    LocalStorage,
    visits:     Document,
    doc_budget: Option<u64>,
    doc_ttl:    Duration,
    /// The retention task's handle; the task holds the store weakly.
    _sweep:     Option<AbortOnDropHandle<()>>,
}

impl Store {
    /// Accepts the blobs, gossip and docs ALPNs on `builder`.
    #[must_use]
    pub fn accept(&self, builder: RouterBuilder) -> RouterBuilder {
        builder
            .accept(
                iroh_blobs::ALPN,
                iroh_blobs::BlobsProtocol::new(self.blob_store(), None),
            )
            .accept(iroh_gossip::ALPN, self.0.gossip.clone())
            .accept(iroh_docs::ALPN, self.0.docs.clone())
    }

    #[must_use]
    pub fn blob_store(&self) -> &BlobStore {
        self.0.blobs.as_ref().as_ref()
    }

    #[must_use]
    pub fn blobs(&self) -> &Blobs {
        self.blob_store().blobs()
    }

    /// This endpoint's gossip instance.
    ///
    /// `iroh_gossip::ALPN` accepts once per router, so this is the only
    /// instance to register.
    #[must_use]
    pub fn gossip(&self) -> &Gossip {
        &self.0.gossip
    }

    /// Mints a namespace this store can write.
    pub async fn create(&self) -> anyhow::Result<Document> {
        self.wrap(self.0.docs.api().create().await?)
    }

    /// Makes `ns` available locally, importing it read-only when it is not
    /// already held.
    ///
    /// A read capability merges into a write capability rather than replacing
    /// it.
    pub async fn open(&self, ns: NamespaceId) -> anyhow::Result<Document> {
        let doc = self
            .0
            .docs
            .api()
            .import_namespace(Capability::Read(ns))
            .await?;
        self.wrap(doc)
    }

    /// Opens the namespace this store's [`LocalStorage`] records at `key`,
    /// minting and recording one on first use.
    pub async fn open_named_doc(&self, key: &str) -> anyhow::Result<Document> {
        self.wrap(open_named_doc(&self.0.docs, &self.0.storage, key).await?)
    }

    /// Leaves the sync set and deletes the replica, its capability and every
    /// entry. Content no other document references is reclaimed with it.
    ///
    /// Fails while anything else holds a [`Document`] for `ns`, after waiting
    /// out a handle already on its way closed.
    pub async fn remove(&self, ns: NamespaceId) -> anyhow::Result<()> {
        let api = self.0.docs.api();

        for _ in 0..REMOVE_ATTEMPTS {
            let doc = api
                .open(ns)
                .await?
                .ok_or_else(|| anyhow::anyhow!("document {ns} is not held"))?;

            // The open above took a handle, so anything past one is somebody
            // else still using the document. `drop_doc` will not refuse on its
            // own, since it closes a handle before removing the replica — the
            // one taken here.
            if doc.status().await?.handles == 1 {
                api.drop_doc(ns).await?;
                // Cleared only once the replica is gone, so a failed remove
                // leaves the recorded age for the next sweep.
                self.0.visits.remove(ns.to_string()).await?;
                return Ok(());
            }

            doc.close().await?;
            n0_future::time::sleep(REMOVE_DELAY).await;
        }

        anyhow::bail!("document {ns} is still held")
    }

    /// Every namespace this store holds, with the capability it holds under.
    pub async fn list(&self) -> anyhow::Result<Vec<(NamespaceId, CapabilityKind)>> {
        let mut stream = self.0.docs.api().list().await?;
        let mut out = Vec::new();
        while let Some(held) = stream.next().await {
            out.push(held?);
        }
        Ok(out)
    }

    /// How long ago each recorded namespace was visited.
    ///
    /// A visit stamped in the future, by a clock since corrected backwards,
    /// reads as age zero rather than as overdue.
    pub async fn visits(&self) -> anyhow::Result<HashMap<NamespaceId, Duration>> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let mut out = HashMap::new();

        for entry in self.0.visits.list(&[""]).await? {
            let Some(ns) = std::str::from_utf8(entry.key())
                .ok()
                .and_then(|key| NamespaceId::from_str(key).ok())
            else {
                continue;
            };
            out.insert(
                ns,
                Duration::from_micros(now.saturating_sub(entry.timestamp())),
            );
        }

        Ok(out)
    }

    /// Stamps `ns` as used now, so retention treats it as still wanted.
    pub async fn record_visit(&self, ns: NamespaceId) -> anyhow::Result<()> {
        self.0.visits.set(ns.to_string(), VISITED).await?;
        Ok(())
    }

    fn wrap(&self, doc: Doc) -> anyhow::Result<Document> {
        Document::new(doc, self.blobs().clone(), self.0.author)
    }
}

/// Opens the namespace recorded under `key`, minting and recording one when
/// absent.
async fn open_named_doc(docs: &Docs, storage: &LocalStorage, key: &str) -> anyhow::Result<Doc> {
    if let Some(doc) = recorded_doc(docs, storage, key).await {
        return Ok(doc);
    }

    let doc = docs.api().create().await?;
    storage.write(key, &doc.id().to_string())?;

    Ok(doc)
}

/// The document recorded at `key`, or `None` if the caller should mint one.
async fn recorded_doc(docs: &Docs, storage: &LocalStorage, key: &str) -> Option<Doc> {
    let ns = match storage.read(key) {
        Ok(Some(text)) => NamespaceId::from_str(text.trim()).ok()?,
        Ok(None) => return None,
        Err(err) => {
            tracing::warn!(%key, ?err, "recorded namespace is unreadable; minting a replacement");
            return None;
        }
    };

    match docs.api().open(ns).await {
        Ok(doc) => doc,
        Err(err) => {
            tracing::warn!(%key, %ns, ?err, "recorded namespace would not open; minting a replacement");
            None
        }
    }
}
