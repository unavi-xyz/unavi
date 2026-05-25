use std::{
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use iroh::{
    Endpoint,
    protocol::{
        DynProtocolHandler,
        RouterBuilder,
    },
};
use iroh_blobs::{
    BlobsProtocol,
    api::Store as BlobStore,
    store::mem::MemStore,
};
use n0_future::task::AbortOnDropHandle;
use parking_lot::RwLock;

use crate::{
    DataStore,
    StoreContext,
    db::Database,
};

pub struct DataStoreBuilder {
    endpoint:  Endpoint,
    gc_timer:  Option<Duration>,
    protocols: Vec<(Vec<u8>, Box<dyn DynProtocolHandler>)>,
    storage:   Storage,
}

pub enum Storage {
    InMemory,
    Path(PathBuf),
}

pub type BoxedRouterBuilder = Box<dyn FnOnce(RouterBuilder) -> RouterBuilder + Send + Sync>;

impl DataStoreBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            gc_timer: None,
            protocols: Vec::new(),
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
    #[must_use]
    pub fn storage_path(mut self, path: PathBuf) -> Self {
        self.storage = Storage::Path(path);
        self
    }

    /// Add a custom protocol handler.
    #[must_use]
    pub fn accept(
        mut self,
        alpn: impl AsRef<[u8]>,
        handler: impl Into<Box<dyn DynProtocolHandler>>,
    ) -> Self {
        self.protocols
            .push((alpn.as_ref().to_vec(), handler.into()));
        self
    }

    /// Build the [`DataStore`].
    pub async fn build(self) -> anyhow::Result<(DataStore, BoxedRouterBuilder)> {
        let (blobs, db) = init_storage(&self.storage).await?;

        let blob_protocol = BlobsProtocol::new(blobs.as_ref().as_ref(), None);

        let ctx = Arc::new(StoreContext {
            blobs,
            connections: scc::HashMap::default(),
            db,
            endpoint: self.endpoint.clone(),
            user_identity: RwLock::new(None),
        });

        let (api_client, api_protocol) = crate::api::protocol(Arc::clone(&ctx));
        let (auth_client, auth_protocol) = crate::auth::protocol(Arc::clone(&ctx));

        let router_builder_fn = Box::new({
            let ctx = Arc::clone(&ctx);
            |builder: RouterBuilder| {
                builder
                    .accept(iroh_blobs::ALPN, blob_protocol)
                    .accept(crate::api::ALPN, api_protocol)
                    .accept(crate::auth::ALPN, auth_protocol)
                    .accept(crate::sync::ALPN, crate::sync::SyncProtocol::new(ctx))
            }
        });

        // Spawn gc task if enabled.
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
                api_client,
                auth_client,
                endpoint: self.endpoint,
                ctx,
                _gc_handle: gc_handle,
            },
            router_builder_fn,
        ))
    }
}

pub type BoxedBlobs = Box<dyn AsRef<BlobStore> + Send + Sync>;

#[cfg(target_family = "wasm")]
async fn init_storage(_storage: &Storage) -> anyhow::Result<(BoxedBlobs, Database)> {
    let blobs = MemStore::new();
    let blobs: BoxedBlobs = Box::new(blobs);

    let db = Database::new_in_memory()?;

    Ok((blobs, db))
}

#[cfg(not(target_family = "wasm"))]
async fn init_storage(storage: &Storage) -> anyhow::Result<(BoxedBlobs, Database)> {
    if let Storage::Path(path) = storage {
        let blob_path = path.join("blob");
        let record_path = path.join("record");
        tokio::fs::create_dir_all(&blob_path).await?;
        tokio::fs::create_dir_all(&record_path).await?;

        let blob_db_path = blob_path.join("blobs.db");
        let blobs = iroh_blobs::store::fs::FsStore::load_with_opts(
            blob_db_path,
            iroh_blobs::store::fs::options::Options::new(&blob_path),
        )
        .await?;
        let blobs: BoxedBlobs = Box::new(blobs);

        let db_path = path.join("index.db");
        let db = Database::new(&db_path)?;

        Ok((blobs, db))
    } else {
        let blobs = MemStore::new();
        let blobs: BoxedBlobs = Box::new(blobs);

        let db = Database::new_in_memory()?;

        Ok((blobs, db))
    }
}
