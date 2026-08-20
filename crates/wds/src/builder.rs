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
    store::mem::MemStore,
};
use iroh_docs::protocol::{
    Builder as DocsBuilder,
    Docs,
};
use iroh_gossip::net::Gossip;
use n0_future::task::AbortOnDropHandle;
use parking_lot::RwLock;

use crate::{
    DataStore,
    StoreContext,
    db::Database,
};

pub struct DataStoreBuilder {
    endpoint: Endpoint,
    gc_timer: Option<Duration>,
    storage:  Storage,
}

pub enum Storage {
    InMemory,
    Path(PathBuf),
}

pub type BoxedRouterBuilder = Box<dyn FnOnce(RouterBuilder) -> RouterBuilder + Send + Sync>;
pub type BoxedBlobs = Box<dyn AsRef<BlobStore> + Send + Sync>;

impl DataStoreBuilder {
    #[must_use]
    pub const fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            gc_timer: None,
            storage: Storage::InMemory,
        }
    }

    /// Spawns a task to run garbage collection at a set frequency.
    /// Disabled by default.
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
        let (blobs, db, docs_builder) = init_storage(&self.storage).await?;
        let blob_store = blobs.as_ref().as_ref().clone();

        let gossip = Gossip::builder().spawn(self.endpoint.clone());

        let docs = docs_builder
            .spawn(self.endpoint.clone(), blob_store.clone(), gossip.clone())
            .await?;

        let blob_protocol = BlobsProtocol::new(&blob_store, None);

        let ctx = Arc::new(StoreContext {
            blobs,
            connections: scc::HashMap::default(),
            db,
            docs: docs.clone(),
            endpoint: self.endpoint.clone(),
            gossip: gossip.clone(),
            hosted: scc::HashMap::default(),
            user_identity: RwLock::new(None),
        });

        let (control_client, control_protocol) = crate::control::protocol(Arc::clone(&ctx));
        let (auth_client, auth_protocol) = crate::auth::protocol(Arc::clone(&ctx));

        let router_builder_fn = Box::new(move |builder: RouterBuilder| {
            builder
                .accept(iroh_blobs::ALPN, blob_protocol)
                .accept(iroh_gossip::ALPN, gossip)
                .accept(iroh_docs::ALPN, docs)
                .accept(crate::control::ALPN, control_protocol)
                .accept(crate::auth::ALPN, auth_protocol)
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
async fn init_storage(storage: &Storage) -> anyhow::Result<(BoxedBlobs, Database, DocsBuilder)> {
    anyhow::ensure!(
        matches!(storage, Storage::InMemory),
        "file storage is not supported on wasm; use Storage::InMemory"
    );
    let blobs: BoxedBlobs = Box::new(MemStore::new());
    let db = Database::new_in_memory()?;
    Ok((blobs, db, Docs::memory()))
}

#[cfg(not(target_family = "wasm"))]
async fn init_storage(storage: &Storage) -> anyhow::Result<(BoxedBlobs, Database, DocsBuilder)> {
    if let Storage::Path(path) = storage {
        let blob_path = path.join("blob");
        let docs_path = path.join("docs");
        tokio::fs::create_dir_all(&blob_path).await?;
        // `Docs::persistent` opens its directory rather than creating it, so
        // the whole layout is laid out here before any store is loaded.
        tokio::fs::create_dir_all(&docs_path).await?;

        let blobs = iroh_blobs::store::fs::FsStore::load_with_opts(
            blob_path.join("blobs.db"),
            iroh_blobs::store::fs::options::Options::new(&blob_path),
        )
        .await?;
        let blobs: BoxedBlobs = Box::new(blobs);

        let db = Database::new(&path.join("index.db"))?;

        Ok((blobs, db, Docs::persistent(docs_path)))
    } else {
        let blobs: BoxedBlobs = Box::new(MemStore::new());
        let db = Database::new_in_memory()?;
        Ok((blobs, db, Docs::memory()))
    }
}
