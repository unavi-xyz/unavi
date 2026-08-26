use std::time::Duration;

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
    protocol::{
        Builder as DocsBuilder,
        Docs,
    },
};
use iroh_gossip::net::Gossip;
use n0_future::task::AbortOnDropHandle;

use crate::local::Storage;

/// Subdirectory of a node's storage the blob and document stores live under.
const STORE_DIR: &str = "store";

pub struct Builder {
    author:   Author,
    endpoint: Endpoint,
    gc_timer: Option<Duration>,
    storage:  Storage,
}

pub type BoxedRouterBuilder = Box<dyn FnOnce(RouterBuilder) -> RouterBuilder + Send + Sync>;
type BoxedBlobs = Box<dyn AsRef<BlobStore> + Send + Sync>;

/// What [`Builder::build`] hands back.
///
/// `guard` has to outlive every use of the rest: dropping it shuts the blob
/// store down and stops garbage collection.
pub struct Store {
    pub blobs:  BlobStore,
    pub docs:   Docs,
    /// The one gossip instance for this endpoint.
    ///
    /// `iroh_gossip::ALPN` can be accepted only once per router, so a second
    /// instance registering it takes every inbound connection from the first,
    /// leaving that one able to dial out and never to receive. Anything that
    /// wants a gossip topic subscribes on this.
    pub gossip: Gossip,
    /// Registers the blob, gossip and docs protocols on a router.
    pub router: BoxedRouterBuilder,
    pub guard:  Guard,
}

pub struct Guard {
    _blobs: BoxedBlobs,
    _gc:    Option<AbortOnDropHandle<()>>,
}

impl Builder {
    #[must_use]
    pub const fn new(endpoint: Endpoint, author: Author) -> Self {
        Self {
            author,
            endpoint,
            gc_timer: None,
            storage: Storage::Ephemeral,
        }
    }

    /// Spawns a task to sweep expired cache tags at a set frequency. Disabled
    /// by default.
    #[must_use]
    pub const fn gc_timer(mut self, frequency: Duration) -> Self {
        self.gc_timer = Some(frequency);
        self
    }

    /// Where blobs and documents are kept. Defaults to
    /// [`Storage::Ephemeral`], which holds both in memory.
    ///
    /// [`Storage::Path`] is not supported on wasm — [`Self::build`] errors if
    /// it is set there, since there is no filesystem to fall back to.
    #[must_use]
    pub fn storage(mut self, storage: Storage) -> Self {
        self.storage = storage;
        self
    }

    pub async fn build(self) -> anyhow::Result<Store> {
        // Blob GC reclaims anything no tag covers; document content carries no
        // tag of its own and is reported through this callback instead. Both
        // halves are wired or neither — a GC run without the callback would
        // reclaim every open document's content.
        let (protect_handler, protect_cb) = ProtectCallbackHandler::new();
        let gc = self.gc_timer.map(|interval| GcConfig {
            interval,
            add_protected: Some(protect_cb),
        });

        let (owned, docs_builder) = init_storage(&self.storage, gc).await?;
        let blobs = owned.as_ref().as_ref().clone();

        sweep_auto_tags(&blobs).await?;

        let gossip = Gossip::builder().spawn(self.endpoint.clone());

        let docs = docs_builder
            .protect_handler(protect_handler)
            .spawn(self.endpoint, blobs.clone(), gossip.clone())
            .await?;

        // Derived rather than minted, so this node writes under the same author
        // every session, and that author names its endpoint.
        let author_id = self.author.id();
        docs.api().author_import(self.author).await?;
        docs.api().author_set_default(author_id).await?;

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

        let gc = self.gc_timer.map(|duration| {
            let blobs = blobs.clone();
            let handle = n0_future::task::spawn(async move {
                loop {
                    // Content itself is reclaimed by the blob store's own
                    // sweep, which reads the tags this pass leaves behind.
                    match crate::cache::sweep(&blobs).await {
                        Ok(deleted) if deleted > 0 => {
                            tracing::debug!(deleted, "expired cache tags");
                        }
                        Ok(_) => {}
                        Err(err) => tracing::warn!(?err, "failed to sweep cache tags"),
                    }
                    n0_future::time::sleep(duration).await;
                }
            });
            AbortOnDropHandle::new(handle)
        });

        Ok(Store {
            blobs,
            docs,
            gossip,
            router,
            guard: Guard {
                _blobs: owned,
                _gc:    gc,
            },
        })
    }
}

// Kept async to match the non-wasm arm's signature, so the call site's
// `.await` works uniformly across targets.
#[cfg(target_family = "wasm")]
#[expect(clippy::unused_async)]
async fn init_storage(
    storage: &Storage,
    gc: Option<GcConfig>,
) -> anyhow::Result<(BoxedBlobs, DocsBuilder)> {
    anyhow::ensure!(
        storage.dir().is_none(),
        "file storage is not supported on wasm"
    );
    let blobs: BoxedBlobs = Box::new(mem_store(gc));
    Ok((blobs, Docs::memory()))
}

#[cfg(not(target_family = "wasm"))]
async fn init_storage(
    storage: &Storage,
    gc: Option<GcConfig>,
) -> anyhow::Result<(BoxedBlobs, DocsBuilder)> {
    if let Some(dir) = storage.dir() {
        let root = dir.join(STORE_DIR);
        let blob_path = root.join("blob");
        let docs_path = root.join("docs");
        tokio::fs::create_dir_all(&blob_path).await?;
        // `Docs::persistent` opens its directory rather than creating it, so
        // the whole layout is laid out here before any store is loaded.
        tokio::fs::create_dir_all(&docs_path).await?;

        let blobs = iroh_blobs::store::fs::FsStore::load_with_opts(
            blob_path.join("blobs.db"),
            iroh_blobs::store::fs::options::Options {
                gc,
                ..iroh_blobs::store::fs::options::Options::new(&blob_path)
            },
        )
        .await?;
        let blobs: BoxedBlobs = Box::new(blobs);

        Ok((blobs, Docs::persistent(docs_path)))
    } else {
        let blobs: BoxedBlobs = Box::new(mem_store(gc));
        Ok((blobs, Docs::memory()))
    }
}

/// Deletes leftover `auto-<rfc3339>` tags minted by bare `add_bytes(..)` calls,
/// which nothing else sweeps; content still referenced by a document survives
/// through the protect callback.
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
