use std::{
    sync::{
        Arc,
        Weak,
    },
    time::Duration,
};

use iroh::Endpoint;
use iroh_blobs::{
    api::Store as BlobStore,
    store::GcConfig,
};
use iroh_docs::{
    Author,
    engine::ProtectCallbackHandler,
    protocol::Docs,
};
use iroh_gossip::net::Gossip;
use n0_future::task::AbortOnDropHandle;
use unavi_local::DeviceStorage;

use crate::{
    Store,
    StoreInner,
    backend::{
        self,
        Backend,
    },
    document::Document,
    error::Result,
    fetch::Fetchers,
    open_named,
    retention,
};

const GIB: u64 = 1024 * 1024 * 1024;

/// The name the visit record is kept under.
const VISITS: &str = "visits";

/// Configures and opens a [`Store`].
pub struct StoreBuilder {
    author:         Author,
    endpoint:       Endpoint,
    sweep_interval: Option<Duration>,
    storage:        DeviceStorage,
    doc_budget:     Option<u64>,
    doc_ttl:        Duration,
}

/// What a load attempt opens from disk.
struct Opened {
    backend: Backend,
    docs:    Docs,
    gossip:  Gossip,
}

impl StoreBuilder {
    #[must_use]
    pub fn new(endpoint: Endpoint, author: Author) -> Self {
        Self {
            author,
            endpoint,
            sweep_interval: None,
            storage: DeviceStorage::memory(),
            doc_budget: Some(8 * GIB),
            doc_ttl: Duration::from_hours(24 * 7),
        }
    }

    /// Runs blob GC and the document retention sweep every `interval`. Neither
    /// runs by default.
    #[must_use]
    pub const fn sweep_interval(mut self, interval: Duration) -> Self {
        self.sweep_interval = Some(interval);
        self
    }

    /// Where blobs and documents are kept. In memory by default.
    #[must_use]
    pub fn storage(mut self, storage: DeviceStorage) -> Self {
        self.storage = storage;
        self
    }

    /// Bytes of read-only documents the retention sweep keeps before evicting
    /// the least recently joined. 8 GiB by default.
    #[must_use]
    pub const fn doc_budget(mut self, bytes: u64) -> Self {
        self.doc_budget = Some(bytes);
        self
    }

    /// How long a document held read-only survives without being joined.
    #[must_use]
    pub const fn doc_ttl(mut self, ttl: Duration) -> Self {
        self.doc_ttl = ttl;
        self
    }

    /// Opens the store. A store on disk that will not load is moved aside, and
    /// an empty one is opened in its place.
    pub async fn build(self) -> Result<Store> {
        let opened = match self.open().await {
            Ok(opened) => opened,
            Err(err) => {
                let Some(moved) = backend::quarantine(&self.storage, &err)? else {
                    return Err(err.into());
                };
                tracing::error!(
                    ?err,
                    moved_to = %moved.display(),
                    "the document store would not load and was moved aside; starting empty"
                );
                self.open().await?
            }
        };
        let Opened {
            backend,
            docs,
            gossip,
        } = opened;
        let blobs = backend.store().clone();

        sweep_auto_tags(&blobs).await?;

        let author = self.author.id();
        docs.api().author_import(self.author).await?;
        docs.api().author_set_default(author).await?;

        let fetchers = Arc::<Fetchers>::default();
        let visits = Document::new(
            open_named(&docs, &self.storage, VISITS).await?,
            blobs.blobs().clone(),
            author,
            Arc::clone(&fetchers),
        )?;
        let downloader = blobs.downloader(&self.endpoint);

        // Weak, so the sweep can be spawned before the store it sweeps exists.
        let store = Store(Arc::new_cyclic(|weak: &Weak<StoreInner>| {
            let sweep = self.sweep_interval.map(|interval| {
                AbortOnDropHandle::new(n0_future::task::spawn(retention::sweep_forever(
                    Weak::clone(weak),
                    interval,
                )))
            });

            StoreInner {
                backend,
                docs,
                gossip,
                downloader,
                author,
                storage: self.storage,
                visits,
                fetchers,
                gate: tokio::sync::RwLock::default(),
                doc_budget: self.doc_budget,
                doc_ttl: self.doc_ttl,
                _sweep: sweep,
            }
        }));

        Ok(store)
    }

    async fn open(&self) -> anyhow::Result<Opened> {
        // Document content carries no tag, so blob GC would reclaim it; the
        // protect callback holds the content of every open document.
        let (protect_handler, protect_cb) = ProtectCallbackHandler::new();
        let gc = self.sweep_interval.map(|interval| GcConfig {
            interval,
            add_protected: Some(protect_cb),
        });

        let (backend, docs_builder) = backend::open(&self.storage, gc).await?;
        let gossip = Gossip::builder().spawn(self.endpoint.clone());
        let docs = docs_builder
            .protect_handler(protect_handler)
            .spawn(
                self.endpoint.clone(),
                backend.store().clone(),
                gossip.clone(),
            )
            .await?;

        Ok(Opened {
            backend,
            docs,
            gossip,
        })
    }
}

/// Deletes the `auto-<rfc3339>` tags a bare `add_bytes` mints; nothing else
/// sweeps them. Document content survives through the protect callback.
async fn sweep_auto_tags(blobs: &BlobStore) -> Result<()> {
    let deleted = blobs
        .tags()
        .delete_prefix("auto-")
        .await
        .map_err(anyhow::Error::from)?;
    if deleted > 0 {
        tracing::info!(deleted, "swept orphaned auto tags");
    }
    Ok(())
}
