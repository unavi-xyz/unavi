use std::{path::Path, path::PathBuf, sync::Arc};

use derive_more::Debug;
use iroh::{Endpoint, EndpointId, protocol::DynProtocolHandler, protocol::Router};
use iroh_blobs::{
    BlobsProtocol,
    store::fs::{FsStore, options::Options},
};
use irpc::Client;
use parking_lot::RwLock;
use tokio::task::JoinError;
use xdid::core::did::Did;

pub use identity::Identity;

pub mod actor;
pub mod api;
mod auth;
mod db;
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
}

/// Builder for [`DataStore`] that allows adding custom protocols to the router.
pub struct DataStoreBuilder {
    path: PathBuf,
    endpoint: Endpoint,
    protocols: Vec<(Vec<u8>, Box<dyn DynProtocolHandler>)>,
}

impl DataStoreBuilder {
    /// Create a new builder.
    pub fn new(path: impl AsRef<Path>, endpoint: Endpoint) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            endpoint,
            protocols: Vec::new(),
        }
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
        let blob_path = self.path.join("blob");
        let record_path = self.path.join("record");
        tokio::fs::create_dir_all(&blob_path).await?;
        tokio::fs::create_dir_all(&record_path).await?;

        let blob_db_path = blob_path.join("blobs.db");
        let blobs = FsStore::load_with_opts(blob_db_path, Options::new(&blob_path)).await?;
        let blob_protocol = BlobsProtocol::new(&blobs, None);

        let db_path = self.path.join("index.db");
        let db = db::Database::new(&db_path).await?;

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

        Ok(DataStore {
            api_client,
            auth_client,
            router,
            ctx,
        })
    }
}

// TODO: Replace session token auth with iroh hooks
type SessionToken = [u8; 32];

#[derive(Debug)]
struct StoreContext {
    blobs: FsStore,
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
    pub fn builder(path: impl AsRef<Path>, endpoint: Endpoint) -> DataStoreBuilder {
        DataStoreBuilder::new(path, endpoint)
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
    pub async fn shutdown(self) -> Result<(), JoinError> {
        self.router.shutdown().await
    }

    /// Returns the database pool. Primarily for testing.
    #[must_use]
    pub fn db(&self) -> &sqlx::Pool<sqlx::Sqlite> {
        self.ctx.db.pool()
    }

    /// Returns the blob store. Primarily for testing.
    #[must_use]
    pub fn blobs(&self) -> &iroh_blobs::store::fs::FsStore {
        &self.ctx.blobs
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
}
