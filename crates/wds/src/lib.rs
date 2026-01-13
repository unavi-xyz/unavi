use std::{path::PathBuf, sync::Arc, time::Duration};

use derive_more::Debug;
use iroh::{Endpoint, EndpointId, protocol::DynProtocolHandler, protocol::Router};
use iroh_blobs::{BlobsProtocol, api::Store as BlobStore, store::mem::MemStore};
use irpc::Client;
use parking_lot::RwLock;
use tokio_util::task::AbortOnDropHandle;
use xdid::core::did::Did;

pub use identity::Identity;

pub mod actor;
pub mod api;
mod auth;
pub mod db;
mod gc;
pub mod identity;
mod quota;
pub mod record;
pub mod signed_bytes;
mod sync;
mod tag;

pub struct DataStore {
    api_client: Client<api::ApiService>,
    auth_client: Client<auth::AuthService>,
    router: Router,
    ctx: Arc<StoreContext>,
    _gc_handle: Option<AbortOnDropHandle<()>>,
}

/// Builder for [`DataStore`] that allows adding custom protocols to the router.
pub struct DataStoreBuilder {
    endpoint: Endpoint,
    gc_timer: Option<Duration>,
    protocols: Vec<(Vec<u8>, Box<dyn DynProtocolHandler>)>,
    storage: Storage,
}

pub enum Storage {
    InMemory,
    Path(PathBuf),
}

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
    ///
    /// # Errors
    ///
    /// Errors if the file system store could not be initialized.
    pub async fn build(self) -> anyhow::Result<DataStore> {
        let (blobs, db) = init_storage(&self.storage).await?;

        let blob_protocol = BlobsProtocol::new(blobs.as_ref().as_ref(), None);

        let ctx = Arc::new(StoreContext {
            blobs,
            connections: scc::HashMap::default(),
            db,
            endpoint: self.endpoint.clone(),
            user_identity: RwLock::new(None),
        });

        let (api_client, api_protocol) = api::protocol(Arc::clone(&ctx));
        let (auth_client, auth_protocol) = auth::protocol(Arc::clone(&ctx));

        let mut router_builder = Router::builder(self.endpoint)
            .accept(iroh_blobs::ALPN, blob_protocol)
            .accept(api::ALPN, api_protocol)
            .accept(auth::ALPN, auth_protocol)
            .accept(sync::ALPN, sync::SyncProtocol::new(Arc::clone(&ctx)));

        // Add custom protocols.
        for (alpn, handler) in self.protocols {
            router_builder = router_builder.accept(alpn, handler);
        }

        let router = router_builder.spawn();

        // Spawn gc task if enabled.
        // TODO: enable for wasm, no handle
        #[cfg(not(target_family = "wasm"))]
        let gc_handle = self.gc_timer.map(|duration| {
            let ctx = Arc::clone(&ctx);
            let handle = tokio::spawn(async move {
                loop {
                    if let Err(err) = ctx.run_gc().await {
                        tracing::error!(?err, "error during garbage collection");
                    }
                    tokio::time::sleep(duration).await;
                }
            });
            AbortOnDropHandle::new(handle)
        });

        #[cfg(target_family = "wasm")]
        let gc_handle = None;

        Ok(DataStore {
            api_client,
            auth_client,
            router,
            ctx,
            _gc_handle: gc_handle,
        })
    }
}

type BoxedBlobs = Box<dyn AsRef<BlobStore> + Send + Sync>;

async fn init_storage(storage: &Storage) -> anyhow::Result<(BoxedBlobs, db::Database)> {
    #[cfg(target_family = "wasm")]
    {
        let blobs = MemStore::new();
        let blobs: BoxedBlobs = Box::new(blobs);

        let db = db::Database::new_in_memory()?;

        Ok((blobs, db))
    }

    #[cfg(not(target_family = "wasm"))]
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
        let db = db::Database::new(&db_path)?;

        Ok((blobs, db))
    } else {
        let blobs = MemStore::new();
        let blobs: BoxedBlobs = Box::new(blobs);

        let db = db::Database::new_in_memory()?;

        Ok((blobs, db))
    }
}

// TODO: Replace session token auth with iroh hooks
type SessionToken = [u8; 32];

#[derive(Debug)]
struct StoreContext {
    #[debug("BlobStore")]
    blobs: BoxedBlobs,
    #[debug("HashMap({})", connections.len())]
    connections: scc::HashMap<SessionToken, ConnectionState>,
    #[debug("Database")]
    db: db::Database,
    #[debug("Endpoint")]
    endpoint: Endpoint,
    #[debug("Option<Identity>")]
    user_identity: RwLock<Option<Arc<Identity>>>,
}

struct ConnectionState {
    /// The authenticated DID of the connection.
    /// Set by the `wds/auth` protocol.
    did: Did,
}

impl DataStore {
    /// Create a new [`DataStoreBuilder`].
    #[must_use]
    pub fn builder(endpoint: Endpoint) -> DataStoreBuilder {
        DataStoreBuilder::new(endpoint)
    }

    #[must_use]
    pub fn endpoint_id(&self) -> EndpointId {
        self.ctx.endpoint.id()
    }

    /// Create an actor targeting the local WDS.
    #[must_use]
    pub fn local_actor(&self, identity: Arc<Identity>) -> actor::Actor {
        actor::Actor::new(
            identity,
            self.router.endpoint().id(),
            self.api_client.clone(),
            self.auth_client.clone(),
        )
    }

    /// Create an actor targeting a remote WDS.
    #[must_use]
    pub fn remote_actor(&self, identity: Arc<Identity>, host: EndpointId) -> actor::Actor {
        let api_client = irpc_iroh::client(self.router.endpoint().clone(), host, api::ALPN);
        let auth_client = irpc_iroh::client(self.router.endpoint().clone(), host, auth::ALPN);
        actor::Actor::new(identity, host, api_client, auth_client)
    }

    /// # Errors
    ///
    /// Errors if protocol tasks could not be joined.
    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.router.shutdown().await?;
        Ok(())
    }

    /// Returns the blob store. Primarily for testing.
    #[must_use]
    pub fn blobs(&self) -> &BlobStore {
        self.ctx.blobs.as_ref().as_ref()
    }

    /// Returns the iroh endpoint. Primarily for testing.
    #[must_use]
    pub fn endpoint(&self) -> &Endpoint {
        &self.ctx.endpoint
    }

    /// Sets the user identity for WDS-to-WDS authentication.
    ///
    /// For embedded/local WDS instances, this allows sync operations to
    /// authenticate to remote stores using the user's signing key instead of
    /// requiring the endpoint to be listed in the DID document.
    pub fn set_user_identity(&self, identity: Arc<Identity>) {
        *self.ctx.user_identity.write() = Some(identity);
    }

    /// Runs garbage collection on the data store.
    /// Cleans up expired pins, orphaned records, and blob tags.
    ///
    /// # Errors
    ///
    /// Errors if database queries fail.
    pub async fn run_gc(&self) -> anyhow::Result<()> {
        self.ctx.run_gc().await
    }

    /// Returns the database. Primarily for testing.
    #[must_use]
    pub fn db(&self) -> &db::Database {
        &self.ctx.db
    }
}
