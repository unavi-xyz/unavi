use std::{
    path::PathBuf,
    sync::Arc,
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
    engine::ProtectCallbackHandler,
    protocol::{
        Builder as DocsBuilder,
        Docs,
    },
};
use iroh_gossip::net::Gossip;
use n0_future::task::AbortOnDropHandle;

use crate::{
    DataStore,
    StoreContext,
    db::Database,
    identity::WdsIdentity,
};

pub struct DataStoreBuilder {
    endpoint:      Endpoint,
    gc_timer:      Option<Duration>,
    identity:      Arc<WdsIdentity>,
    serve_control: bool,
    storage:       Storage,
}

pub enum Storage {
    InMemory,
    Path(PathBuf),
}

pub type BoxedRouterBuilder = Box<dyn FnOnce(RouterBuilder) -> RouterBuilder + Send + Sync>;
pub type BoxedBlobs = Box<dyn AsRef<BlobStore> + Send + Sync>;

impl DataStoreBuilder {
    #[must_use]
    pub const fn new(endpoint: Endpoint, identity: Arc<WdsIdentity>) -> Self {
        Self {
            endpoint,
            gc_timer: None,
            identity,
            serve_control: false,
            storage: Storage::InMemory,
        }
    }

    /// Answers `wds/control`, letting other peers ask this node to host their
    /// documents and pin their blobs. Disabled by default.
    ///
    /// Performs authentication but no authorization: any peer with a resolvable
    /// DID gets service up to a default quota minted on demand. Only intended
    /// storage hosts should enable it. Peer sync is unaffected, running over
    /// `iroh_docs::ALPN` regardless.
    #[must_use]
    pub const fn serve_control(mut self) -> Self {
        self.serve_control = true;
        self
    }

    /// Spawns a task to run garbage collection at a set frequency, for both the
    /// quota ledger and the blob store. Disabled by default.
    #[must_use]
    pub const fn gc_timer(mut self, frequency: Duration) -> Self {
        self.gc_timer = Some(frequency);
        self
    }

    /// Specify a directory path for file storage.
    /// If not provided, defaults to in-memory storage.
    ///
    /// Not supported on wasm — [`Self::build`] errors if this is set there,
    /// since there is no filesystem to fall back to.
    #[must_use]
    pub fn storage_path(mut self, path: PathBuf) -> Self {
        self.storage = Storage::Path(path);
        self
    }

    /// Build the [`DataStore`].
    pub async fn build(self) -> anyhow::Result<(DataStore, BoxedRouterBuilder)> {
        // Blob GC reclaims anything no tag covers; document content carries no
        // tag of its own and is reported through this callback instead. Both
        // halves are wired or neither — a GC run without the callback would
        // reclaim every open document's content.
        let (protect_handler, protect_cb) = ProtectCallbackHandler::new();
        let gc = self.gc_timer.map(|interval| GcConfig {
            interval,
            add_protected: Some(protect_cb),
        });

        let (blobs, db, docs_builder) = init_storage(&self.storage, gc).await?;
        let blob_store = blobs.as_ref().as_ref().clone();

        sweep_auto_tags(&blob_store).await?;

        let gossip = Gossip::builder().spawn(self.endpoint.clone());

        let docs = docs_builder
            .protect_handler(protect_handler)
            .spawn(self.endpoint.clone(), blob_store.clone(), gossip.clone())
            .await?;

        // Derived rather than minted, so this node writes under the same author
        // every session, and that author names its endpoint.
        let author = self.identity.author();
        let author_id = author.id();
        docs.api().author_import(author).await?;
        docs.api().author_set_default(author_id).await?;

        let blob_protocol = BlobsProtocol::new(&blob_store, None);

        let ctx = Arc::new(StoreContext {
            blobs,
            connections: scc::HashMap::default(),
            db,
            docs: docs.clone(),
            endpoint: self.endpoint.clone(),
            gossip: gossip.clone(),
            hosted: scc::HashMap::default(),
            identity: self.identity,
        });

        let (control_client, control_protocol) = crate::control::protocol(Arc::clone(&ctx));
        let (auth_client, auth_protocol) = crate::auth::protocol(Arc::clone(&ctx));

        // Auth is served regardless: it mints the session tokens the registry's
        // own control plane validates, and grants nothing on its own.
        let serve_control = self.serve_control;
        let router_builder_fn = Box::new(move |builder: RouterBuilder| {
            let builder = builder
                .accept(iroh_blobs::ALPN, blob_protocol)
                .accept(iroh_gossip::ALPN, gossip)
                .accept(iroh_docs::ALPN, docs)
                .accept(crate::auth::ALPN, auth_protocol);
            if serve_control {
                builder.accept(crate::control::ALPN, control_protocol)
            } else {
                builder
            }
        });

        let gc_handle = self.gc_timer.map(|duration| {
            let ctx = Arc::clone(&ctx);
            let handle = n0_future::task::spawn(async move {
                loop {
                    if let Err(err) = ctx.run_gc().await {
                        tracing::error!(?err, "error during garbage collection");
                    }
                    n0_future::time::sleep(duration).await;
                }
            });
            AbortOnDropHandle::new(handle)
        });

        Ok((
            DataStore {
                control_client,
                auth_client,
                endpoint: self.endpoint,
                ctx,
                _gc_handle: gc_handle,
            },
            router_builder_fn,
        ))
    }
}

// Kept async to match the non-wasm arm's signature, so the call site's
// `.await` works uniformly across targets.
#[cfg(target_family = "wasm")]
#[expect(clippy::unused_async)]
async fn init_storage(
    storage: &Storage,
    gc: Option<GcConfig>,
) -> anyhow::Result<(BoxedBlobs, Database, DocsBuilder)> {
    anyhow::ensure!(
        matches!(storage, Storage::InMemory),
        "file storage is not supported on wasm; use Storage::InMemory"
    );
    let blobs: BoxedBlobs = Box::new(mem_store(gc));
    let db = Database::new_in_memory()?;
    Ok((blobs, db, Docs::memory()))
}

#[cfg(not(target_family = "wasm"))]
async fn init_storage(
    storage: &Storage,
    gc: Option<GcConfig>,
) -> anyhow::Result<(BoxedBlobs, Database, DocsBuilder)> {
    if let Storage::Path(path) = storage {
        let blob_path = path.join("blob");
        let docs_path = path.join("docs");
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

        let db = Database::new(&path.join("index.db"))?;

        Ok((blobs, db, Docs::persistent(docs_path)))
    } else {
        let blobs: BoxedBlobs = Box::new(mem_store(gc));
        let db = Database::new_in_memory()?;
        Ok((blobs, db, Docs::memory()))
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
