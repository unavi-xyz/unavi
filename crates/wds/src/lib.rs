use std::{path::Path, sync::Arc};

use derive_more::Debug;
use iroh::{Endpoint, EndpointId, endpoint::RelayMode, protocol::Router};
use iroh_blobs::{
    BlobsProtocol,
    store::fs::{FsStore, options::Options},
};
use irpc::Client;
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

/// Wired data store.
pub struct DataStore {
    api_client: Client<api::ApiService>,
    auth_client: Client<auth::AuthService>,
    router: Router,
    ctx: Arc<StoreContext>,
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
}

struct ConnectionState {
    /// The authenticated DID of the connection.
    /// Set by the `wds/auth` protocol.
    did: Did,
}

impl DataStore {
    /// # Errors
    ///
    /// Errors if the iroh endpoint could not be constructed, or the file system
    /// store could not be initialized.
    pub async fn new(path: &Path) -> anyhow::Result<Self> {
        let endpoint = Endpoint::builder().bind().await?;
        Self::with_endpoint(path, endpoint).await
    }

    /// Empty endpoint constructor with minimal networking (no relay/STUN/pkarr).
    ///
    /// # Errors
    ///
    /// Errors if the iroh endpoint could not be constructed, or the file system
    /// store could not be initialized.
    pub async fn new_empty(path: &Path) -> anyhow::Result<Self> {
        let endpoint = Endpoint::empty_builder(RelayMode::Disabled).bind().await?;
        Self::with_endpoint(path, endpoint).await
    }

    async fn with_endpoint(path: &Path, endpoint: Endpoint) -> anyhow::Result<Self> {
        let blob_path = path.join("blob");
        let record_path = path.join("record");
        tokio::fs::create_dir_all(&blob_path).await?;
        tokio::fs::create_dir_all(&record_path).await?;

        let blob_db_path = blob_path.join("blobs.db");
        let blobs = FsStore::load_with_opts(blob_db_path, Options::new(&blob_path)).await?;
        let blob_protocol = BlobsProtocol::new(&blobs, None);

        let db_path = path.join("index.db");
        let db = db::Database::new(&db_path).await?;

        let ctx = Arc::new(StoreContext {
            blobs,
            connections: scc::HashMap::default(),
            db,
            endpoint: endpoint.clone(),
        });

        let (api_client, api_protocol) = api::protocol(Arc::clone(&ctx));
        let (auth_client, auth_protocol) = auth::protocol(Arc::clone(&ctx));

        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, blob_protocol)
            .accept(api::ALPN, api_protocol)
            .accept(auth::ALPN, auth_protocol)
            .accept(sync::ALPN, sync::SyncProtocol::new(Arc::clone(&ctx)))
            .spawn();

        Ok(Self {
            api_client,
            auth_client,
            router,
            ctx,
        })
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
}
