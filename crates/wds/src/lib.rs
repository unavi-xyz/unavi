//! # Wired Data Store (WDS)

use std::{
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

const VISITED: &[u8] = b"1";

/// A released [`Document`] closes on a task spawned as it drops, so a replica
/// nothing uses can still read as held for a moment. A drop waits that out
/// rather than reporting a document as busy.
///
/// The wait covers one round trip to the docs actor, not a document in use —
/// a sweep passing over held documents should give up on each of them quickly.
const DROP_ATTEMPTS: usize = 10;
const DROP_DELAY: Duration = Duration::from_millis(20);

pub type BoxedRouterBuilder = Box<dyn FnOnce(RouterBuilder) -> RouterBuilder + Send + Sync>;

/// The concrete blob store the [`BlobStore`] client handles talk to. Boxed
/// because which one it is depends on the target and the configured storage.
trait OwnedBlobs: AsRef<BlobStore> + std::fmt::Debug + Send + Sync {}
impl<T: AsRef<BlobStore> + std::fmt::Debug + Send + Sync> OwnedBlobs for T {}
type BoxedBlobs = Box<dyn OwnedBlobs>;

/// This node's blobs and documents. Clones share one store; dropping the last
/// of them shuts it down and stops its retention sweep.
#[derive(Clone, Debug)]
pub struct Store(Arc<Inner>);

#[derive(Debug)]
struct Inner {
    blobs:      BoxedBlobs,
    docs:       Docs,
    gossip:     Gossip,
    author:     AuthorId,
    storage:    LocalStorage,
    visits:     Document,
    doc_budget: Option<u64>,
    doc_ttl:    Duration,
    /// The sweep holds this weakly, so it neither keeps the store alive nor
    /// outlives it.
    _sweep:     Option<AbortOnDropHandle<()>>,
}

impl Store {
    /// Makes `ns` available locally, importing it read-only if this node does
    /// not already hold it.
    ///
    /// Merging a read capability into a write capability already held is a
    /// no-op, not a downgrade.
    pub async fn open(&self, ns: NamespaceId) -> anyhow::Result<Document> {
        let doc = self
            .0
            .docs
            .api()
            .import_namespace(Capability::Read(ns))
            .await?;
        self.wrap(doc)
    }

    /// Mints a namespace this node holds the write capability for.
    pub async fn create(&self) -> anyhow::Result<Document> {
        self.wrap(self.0.docs.api().create().await?)
    }

    /// Opens the namespace this store's [`LocalStorage`] records at `key`,
    /// minting and recording one on first use.
    pub async fn open_named_doc(&self, key: &str) -> anyhow::Result<Document> {
        self.wrap(open_named_doc(&self.0.docs, &self.0.storage, key).await?)
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
    /// `iroh_gossip::ALPN` can be accepted only once per router. A second
    /// instance registering it takes every inbound connection from the first,
    /// leaving that one able to dial out but never to receive.
    #[must_use]
    pub fn gossip(&self) -> &Gossip {
        &self.0.gossip
    }

    /// How many bytes of held documents this device tolerates before the
    /// oldest-visited read-only ones are evicted, or `None` for no budget.
    #[must_use]
    fn doc_budget(&self) -> Option<u64> {
        self.0.doc_budget
    }

    /// How long a read-only document survives unvisited.
    #[must_use]
    fn doc_ttl(&self) -> Duration {
        self.0.doc_ttl
    }

    /// Every namespace this node holds, and the capability it holds it under.
    pub async fn list(&self) -> anyhow::Result<Vec<(NamespaceId, CapabilityKind)>> {
        let mut stream = self.0.docs.api().list().await?;
        let mut out = Vec::new();
        while let Some(held) = stream.next().await {
            out.push(held?);
        }
        Ok(out)
    }

    /// Records that `ns` was used now, so retention can tell a document this
    /// node still wants from one it happens to be holding.
    pub async fn record_visit(&self, ns: NamespaceId) -> anyhow::Result<()> {
        self.0.visits.set(ns.to_string(), VISITED).await?;
        Ok(())
    }

    /// How long ago each recorded document was visited.
    ///
    /// A visit stamped in the future, by a clock that has since been corrected
    /// backwards, reads as an age of zero rather than as a document overdue
    /// for eviction.
    pub async fn visits(&self) -> anyhow::Result<Vec<(NamespaceId, Duration)>> {
        let now = n0_future::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let mut out = Vec::new();

        for entry in self.0.visits.list(&[""]).await? {
            let Some(ns) = std::str::from_utf8(entry.key())
                .ok()
                .and_then(|key| NamespaceId::from_str(key).ok())
            else {
                continue;
            };
            out.push((
                ns,
                Duration::from_micros(now.saturating_sub(entry.timestamp())),
            ));
        }

        Ok(out)
    }

    /// Leaves the sync set and deletes the replica, its capability and every
    /// entry it holds. Content no other document references falls out of every
    /// garbage-collection root with it.
    ///
    /// Fails while anything else still holds a [`Document`] for `ns`, having
    /// first waited out a handle already on its way closed.
    pub async fn drop(&self, ns: NamespaceId) -> anyhow::Result<()> {
        self.0.visits.remove(ns.to_string()).await?;
        let api = self.0.docs.api();

        for _ in 0..DROP_ATTEMPTS {
            let doc = api
                .open(ns)
                .await?
                .ok_or_else(|| anyhow::anyhow!("document {ns} is not held"))?;

            // The open above took a handle, so anything past one is somebody
            // else still using the document. `drop_doc` will not refuse on its
            // own, since it closes a handle before removing the replica — the
            // one taken here.
            if doc.status().await?.handles == 1 {
                return api.drop_doc(ns).await;
            }

            doc.close().await?;
            n0_future::time::sleep(DROP_DELAY).await;
        }

        anyhow::bail!("document {ns} is still held")
    }

    fn wrap(&self, doc: Doc) -> anyhow::Result<Document> {
        Document::new(doc, self.blobs().clone(), self.0.author)
    }
}

/// Opens a named document, persisting the [`NamespaceId`] in [`LocalStorage`]
/// at the given `key`.
async fn open_named_doc(docs: &Docs, storage: &LocalStorage, key: &str) -> anyhow::Result<Doc> {
    let ns = match storage.read(key) {
        Ok(Some(text)) => NamespaceId::from_str(text.trim()).ok(),
        Ok(None) => None,
        Err(err) => {
            tracing::warn!(%key, ?err, "recorded namespace is unreadable; minting a replacement");
            None
        }
    };

    if let Some(ns) = ns {
        let doc = match docs.api().open(ns).await {
            Ok(doc) => doc,
            Err(err) => {
                tracing::warn!(%ns, ?err, "recorded namespace is unreadable; minting a replacement");
                None
            }
        };

        if let Some(doc) = doc {
            return Ok(doc);
        }
    }

    let doc = docs.api().create().await?;
    storage.write(key, &doc.id().to_string())?;

    Ok(doc)
}
