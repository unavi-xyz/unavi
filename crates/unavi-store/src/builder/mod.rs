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
    Store,
    StoreInner,
    document::Document,
    open_named_doc,
    retention,
};

#[cfg(not(target_family = "wasm"))] mod fs;
#[cfg(target_family = "wasm")] mod web;

const VISITS_KEY: &str = "visits.bin";

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
            doc_ttl: Duration::from_hours(24 * 7),
        }
    }

    /// Runs blob GC and the document retention sweep at `frequency`. No sweep
    /// by default.
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

    /// Byte budget for held documents; unbounded by default.
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

    pub async fn build(self) -> anyhow::Result<Store> {
        // Document content carries no tag, so blob GC would reclaim it; the
        // protect callback holds the content of every open document.
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

        // Weak, so the sweep can be spawned before the store it sweeps exists.
        let store = Store(Arc::new_cyclic(|weak: &Weak<StoreInner>| {
            let sweep = self.gc_timer.map(|interval| {
                AbortOnDropHandle::new(n0_future::task::spawn(retention::sweep_forever(
                    Weak::clone(weak),
                    interval,
                )))
            });

            StoreInner {
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

        Ok(store)
    }
}

/// Deletes the `auto-<rfc3339>` tags a bare `add_bytes` mints; nothing else
/// sweeps them. Document content survives through the protect callback.
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
