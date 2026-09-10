use std::{
    sync::{
        Arc,
        Weak,
    },
    time::Duration,
};

use iroh::{
    Endpoint,
    protocol::RouterBuilder,
};
use iroh_blobs::{
    BlobsProtocol,
    api::Store as BlobStore,
    store::{
        GcConfig,
        mem::MemStore,
    },
};
use iroh_docs::{
    Author,
    engine::ProtectCallbackHandler,
};
use iroh_gossip::net::Gossip;
use n0_future::task::AbortOnDropHandle;
use unavi_local::LocalStorage;

use crate::{
    BoxedBlobs,
    BoxedRouterBuilder,
    Inner,
    Store,
    document::Document,
    open_named_doc,
    retention,
};

#[cfg(not(target_family = "wasm"))] mod fs;
#[cfg(target_family = "wasm")] mod web;

const VISITS_KEY: &str = "visits.bin";

/// A store, and the one-shot registration of the protocols it answers on.
pub struct Spawned {
    pub store:  Store,
    pub router: BoxedRouterBuilder,
}

pub struct StoreBuilder {
    author:     Author,
    endpoint:   Endpoint,
    gc_timer:   Option<Duration>,
    storage:    LocalStorage,
    doc_budget: Option<u64>,
    doc_ttl:    Duration,
}

impl StoreBuilder {
    #[must_use]
    pub fn new(endpoint: Endpoint, author: Author) -> Self {
        Self {
            author,
            endpoint,
            gc_timer: None,
            storage: LocalStorage::default(),
            doc_budget: None,
            doc_ttl: Duration::from_hours(7 * 24),
        }
    }

    /// Sweeps at a set frequency: blobs no document or tag covers, and
    /// documents past the retention this builder sets. Disabled by default.
    #[must_use]
    pub const fn gc_timer(mut self, frequency: Duration) -> Self {
        self.gc_timer = Some(frequency);
        self
    }

    /// Where blobs and documents are kept.
    #[must_use]
    pub fn storage(mut self, storage: LocalStorage) -> Self {
        self.storage = storage;
        self
    }

    /// How many bytes of held documents this device tolerates. Unbounded by
    /// default.
    #[must_use]
    pub const fn doc_budget(mut self, bytes: u64) -> Self {
        self.doc_budget = Some(bytes);
        self
    }

    /// How long a document held read-only survives unvisited.
    #[must_use]
    pub const fn doc_ttl(mut self, ttl: Duration) -> Self {
        self.doc_ttl = ttl;
        self
    }

    pub async fn build(self) -> anyhow::Result<Spawned> {
        // Blob GC reclaims anything no tag covers. Document content carries no
        // tag of its own, so without this callback a GC run reclaims every open
        // document's content.
        let (protect_handler, protect_cb) = ProtectCallbackHandler::new();
        let gc = self.gc_timer.map(|interval| GcConfig {
            interval,
            add_protected: Some(protect_cb),
        });

        let (owned, docs_builder) = cfg_select! {
            target_family = "wasm" => web::init(gc)?,
            _ => fs::init(&self.storage, gc).await?,
        };
        let blobs = owned.as_ref().as_ref().clone();

        sweep_auto_tags(&blobs).await?;

        let gossip = Gossip::builder().spawn(self.endpoint.clone());

        let docs = docs_builder
            .protect_handler(protect_handler)
            .spawn(self.endpoint, blobs.clone(), gossip.clone())
            .await?;

        let author = self.author.id();
        docs.api().author_import(self.author).await?;
        docs.api().author_set_default(author).await?;

        let visits = Document::new(
            open_named_doc(&docs, &self.storage, VISITS_KEY).await?,
            blobs.blobs().clone(),
            author,
        )?;

        let blob_protocol = BlobsProtocol::new(&blobs, None);

        let router = {
            let docs = docs.clone();
            let gossip = gossip.clone();
            Box::new(move |builder: RouterBuilder| {
                builder
                    .accept(iroh_blobs::ALPN, blob_protocol)
                    .accept(iroh_gossip::ALPN, gossip)
                    .accept(iroh_docs::ALPN, docs)
            })
        };

        // The sweep is handed a weak reference so that it can be spawned before
        // the store it sweeps exists, and so that it never keeps that store
        // alive past its last handle.
        let store = Store(Arc::new_cyclic(|weak: &Weak<Inner>| {
            let sweep = self.gc_timer.map(|interval| {
                AbortOnDropHandle::new(n0_future::task::spawn(retention::sweep_forever(
                    Weak::clone(weak),
                    interval,
                )))
            });

            Inner {
                blobs: owned,
                docs,
                gossip,
                author,
                storage: self.storage,
                visits,
                doc_budget: self.doc_budget,
                doc_ttl: self.doc_ttl,
                _sweep: sweep,
            }
        }));

        Ok(Spawned { store, router })
    }
}

/// Deletes the `auto-<rfc3339>` tags a bare `add_bytes` mints, which nothing
/// else sweeps. Content a document still references survives through the
/// protect callback.
async fn sweep_auto_tags(blobs: &BlobStore) -> anyhow::Result<()> {
    let deleted = blobs.tags().delete_prefix("auto-").await?;
    if deleted > 0 {
        tracing::info!(deleted, "swept orphaned auto tags");
    }
    Ok(())
}

fn mem_store(gc: Option<GcConfig>) -> MemStore {
    MemStore::new_with_opts(iroh_blobs::store::mem::Options { gc_config: gc })
}
