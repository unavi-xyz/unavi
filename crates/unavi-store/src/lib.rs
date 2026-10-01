//! This node's document and blob store, backed by `iroh-docs` and `iroh-blobs`.
//!
//! A [`Document`] is a key-value map identified by a namespace id. Each key is
//! a separately signed entry, so peers writing different keys merge and peers
//! writing the same key resolve by timestamp. An entry stores the hash of its
//! value, so a value repeated across keys is stored once.
//!
//! Content synced from peers is fetched here rather than by the docs engine,
//! within [`MAX_ENTRY_BYTES`] per entry and [`MAX_DOC_BYTES`] per document. A
//! document arrives through [`Store::join`], which also records the visit that
//! retention ages it by.

use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use iroh::{
    EndpointAddr,
    protocol::RouterBuilder,
};
use iroh_blobs::api::{
    Store as BlobStore,
    blobs::Blobs,
    downloader::Downloader,
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
use tokio::sync::RwLock;
use unavi_local::DeviceStorage;

use crate::{
    backend::Backend,
    fetch::Fetchers,
};
pub use crate::{
    builder::StoreBuilder,
    document::{
        Document,
        Events,
        Tombstones,
    },
    error::{
        Error,
        Result,
    },
    fetch::{
        MAX_DOC_BYTES,
        MAX_DOC_ENTRIES,
        MAX_ENTRY_BYTES,
    },
};

mod backend;
mod builder;
mod document;
mod error;
mod fetch;
mod retention;

/// Placeholder non-empty value for marking a document as visited.
const VISITED: &[u8] = b"1";

/// A released [`Document`] closes on a spawned task, so a replica nothing uses
/// can still read as held for a moment.
const REMOVE_ATTEMPTS: usize = 10;
const REMOVE_DELAY: Duration = Duration::from_millis(20);

/// A cloneable handle to the store.
#[derive(Clone, Debug)]
pub struct Store(Arc<StoreInner>);

#[derive(Debug)]
struct StoreInner {
    backend:    Backend,
    docs:       Docs,
    gossip:     Gossip,
    downloader: Downloader,
    author:     AuthorId,
    storage:    DeviceStorage,
    visits:     Document,
    fetchers:   Arc<Fetchers>,
    /// Taken shared by every open and exclusively by a remove, so a remove
    /// never deletes a replica an open is handing out.
    gate:       RwLock<()>,
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
        self.0.backend.store()
    }

    #[must_use]
    pub fn blobs(&self) -> &Blobs {
        self.blob_store().blobs()
    }

    /// Pulls blobs from named providers. Unbounded: a caller bounds what it
    /// asks for.
    #[must_use]
    pub fn downloader(&self) -> &Downloader {
        &self.0.downloader
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
    pub async fn create(&self) -> Result<Document> {
        let doc = self.0.docs.api().create().await?;
        self.wrap(doc)
    }

    /// `ns` if this store already holds it. Never imports.
    pub async fn held(&self, ns: NamespaceId) -> Result<Option<Document>> {
        let _gate = self.0.gate.read().await;
        let Some(doc) = open_held(&self.0.docs, ns).await? else {
            return Ok(None);
        };
        self.wrap(doc).map(Some)
    }

    /// Opens `ns`, importing it read-only if it is not held, and syncs it with
    /// `peers`.
    ///
    /// Records a visit, so retention ages the document from now. The returned
    /// events are subscribed before the sync starts, so they see every entry it
    /// brings. Content is fetched within this crate's caps, from the peer that
    /// sent each entry and from every peer any join named.
    pub async fn join(
        &self,
        ns: NamespaceId,
        peers: Vec<EndpointAddr>,
    ) -> Result<(Document, Events)> {
        let doc = self.import(ns).await?;
        self.record_visit(ns).await?;
        doc.refuse_downloads().await?;

        let providers = peers.iter().map(|peer| peer.id).collect::<Vec<_>>();
        if self.0.fetchers.contains(ns) {
            self.0.fetchers.add_providers(ns, &providers);
        } else {
            let engine = doc.doc().subscribe().await?;
            let scan = doc.scan().await?;
            self.0.fetchers.start(
                ns,
                engine,
                scan,
                &providers,
                fetch::Context {
                    blobs:      self.blobs().clone(),
                    downloader: self.0.downloader.clone(),
                    docs:       self.0.docs.api().clone(),
                },
            );
        }

        let events = doc.subscribe().await?;
        doc.doc().start_sync(peers).await?;

        Ok((doc, events))
    }

    /// The document this device records under `name`, minting and recording
    /// one on first use.
    pub async fn named(&self, name: &str) -> Result<Document> {
        let _gate = self.0.gate.read().await;
        let doc = open_named(&self.0.docs, &self.0.storage, name).await?;
        self.wrap(doc)
    }

    /// Leaves the sync set and deletes the replica, its capability and every
    /// entry. Content no other document references is reclaimed with it.
    ///
    /// Fails while anything else holds a [`Document`] for `ns`, after waiting
    /// out a handle already on its way closed.
    pub async fn remove(&self, ns: NamespaceId) -> Result<()> {
        let api = self.0.docs.api();

        for _ in 0..REMOVE_ATTEMPTS {
            {
                let _gate = self.0.gate.write().await;
                let doc = open_held(&self.0.docs, ns)
                    .await?
                    .ok_or(Error::NotHeld(ns))?;

                // The open above took a handle, so anything past one is
                // somebody else still using the document. `drop_doc` will not
                // refuse on its own, since it closes a handle before removing
                // the replica — the one taken here.
                if doc.status().await?.handles == 1 {
                    self.0.fetchers.stop(ns);
                    api.drop_doc(ns).await?;
                    // Cleared only once the replica is gone, so a failed remove
                    // leaves the recorded age for the next sweep.
                    self.0.visits.remove_key(ns.to_string()).await?;
                    return Ok(());
                }

                doc.close().await?;
            }
            n0_future::time::sleep(REMOVE_DELAY).await;
        }

        Err(Error::StillHeld(ns))
    }

    /// Every namespace this store holds, with the capability it holds under.
    pub async fn list(&self) -> Result<Vec<(NamespaceId, CapabilityKind)>> {
        let mut stream = self.0.docs.api().list().await?;
        let mut out = Vec::new();
        while let Some(held) = stream.next().await {
            out.push(held?);
        }
        Ok(out)
    }

    /// How long ago each joined namespace was last joined.
    ///
    /// A visit stamped in the future, by a clock since corrected backwards,
    /// reads as age zero rather than as overdue.
    pub async fn visits(&self) -> Result<HashMap<NamespaceId, Duration>> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let mut out = HashMap::new();

        for entry in self.0.visits.list("").await? {
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

    /// Imports `ns` read-only unless held. A read capability merges into a
    /// held write capability rather than replacing it.
    async fn import(&self, ns: NamespaceId) -> Result<Document> {
        let _gate = self.0.gate.read().await;
        let doc = self
            .0
            .docs
            .api()
            .import_namespace(Capability::Read(ns))
            .await?;
        self.wrap(doc)
    }

    async fn record_visit(&self, ns: NamespaceId) -> Result<()> {
        self.0.visits.set(ns.to_string(), VISITED).await?;
        Ok(())
    }

    fn wrap(&self, doc: Doc) -> Result<Document> {
        Document::new(
            doc,
            self.blobs().clone(),
            self.0.author,
            Arc::clone(&self.0.fetchers),
        )
    }
}

/// `ns`, or `None` when the store does not hold it.
///
/// Opening a namespace the store lacks fails like any other error, so a failed
/// open is told apart by whether the namespace is listed.
async fn open_held(docs: &Docs, ns: NamespaceId) -> anyhow::Result<Option<Doc>> {
    let err = match docs.api().open(ns).await {
        Ok(doc) => return Ok(doc),
        Err(err) => err,
    };

    let mut held = docs.api().list().await?;
    while let Some(entry) = held.next().await {
        if entry?.0 == ns {
            return Err(err);
        }
    }
    Ok(None)
}

/// Opens the document recorded under `name`, minting and recording one when
/// absent.
async fn open_named(docs: &Docs, storage: &DeviceStorage, name: &str) -> Result<Doc> {
    let key = format!("docs/{name}");

    if let Some(doc) = recorded_doc(docs, storage, &key).await {
        return Ok(doc);
    }
    if let Some(doc) = legacy_doc(docs, storage, name).await {
        storage.write(&key, &doc.id().to_string())?;
        return Ok(doc);
    }

    let doc = docs.api().create().await?;
    storage.write(&key, &doc.id().to_string())?;

    Ok(doc)
}

/// A document recorded where builds before `docs/` recorded it: at the bare
/// name, or with a `.bin` suffix. Read so an upgrade keeps its documents.
async fn legacy_doc(docs: &Docs, storage: &DeviceStorage, name: &str) -> Option<Doc> {
    for key in [name.to_owned(), format!("{name}.bin")] {
        if let Some(doc) = recorded_doc(docs, storage, &key).await {
            return Some(doc);
        }
    }
    None
}

/// The document recorded at `key`, or `None` if the caller should mint one.
async fn recorded_doc(docs: &Docs, storage: &DeviceStorage, key: &str) -> Option<Doc> {
    let ns = match storage.read(key) {
        Ok(Some(text)) => NamespaceId::from_str(text.trim()).ok()?,
        Ok(None) => return None,
        Err(err) => {
            tracing::warn!(%key, ?err, "recorded namespace is unreadable; minting a replacement");
            return None;
        }
    };

    match open_held(docs, ns).await {
        Ok(doc) => doc,
        Err(err) => {
            tracing::warn!(%key, %ns, ?err, "recorded namespace would not open; minting a replacement");
            None
        }
    }
}
