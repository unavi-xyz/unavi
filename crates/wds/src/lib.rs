use std::{path::Path, sync::Arc};

use blake3::Hash;
use iroh::{Endpoint, EndpointId, protocol::Router};
use iroh_blobs::{
    BlobsProtocol,
    store::fs::{FsStore, options::Options},
};
use irpc::Client;
use tokio::task::JoinError;
use xdid::{core::did::Did, methods::key::p256::P256KeyPair};

pub mod actor;
pub mod api;
mod auth;
mod blob;
mod db;
mod gc;
mod quota;
pub mod signed_bytes;
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

struct StoreContext {
    blobs: FsStore,
    connections: scc::HashMap<SessionToken, ConnectionState>,
    db: db::Database,
    endpoint_id: EndpointId,
    logs: p2panda_store::SqliteStore<Hash, ()>,
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

        let blob_path = path.join("blob");
        let record_path = path.join("record");
        tokio::fs::create_dir_all(&blob_path).await?;
        tokio::fs::create_dir_all(&record_path).await?;

        let blob_db_path = blob_path.join("blobs.db");
        let blobs = FsStore::load_with_opts(blob_db_path, Options::new(&blob_path)).await?;
        let blob_protocol = BlobsProtocol::new(&blobs, None);

        let db_path = path.join("index.db");
        let db = db::Database::new(&db_path).await?;

        let logs = p2panda_store::SqliteStore::new(db.pool().clone());

        let ctx = Arc::new(StoreContext {
            blobs,
            connections: scc::HashMap::default(),
            db,
            endpoint_id: endpoint.id(),
            logs,
        });

        let (api_client, api_protocol) = api::protocol(Arc::clone(&ctx));
        let (auth_client, auth_protocol) = auth::protocol(Arc::clone(&ctx));

        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, blob_protocol)
            .accept(api::ALPN, api_protocol)
            .accept(auth::ALPN, auth_protocol)
            .spawn();

        Ok(Self {
            api_client,
            auth_client,
            router,
            ctx,
        })
    }

    #[must_use]
    pub fn actor(&self, did: Did, signing_key: P256KeyPair) -> actor::Actor {
        actor::Actor::new(
            did,
            signing_key,
            self.router.endpoint().id(),
            self.api_client.clone(),
            self.auth_client.clone(),
        )
    }

    /// # Errors
    ///
    /// Errors if protocol tasks could not be joined.
    pub async fn shutdown(self) -> Result<(), JoinError> {
        self.router.shutdown().await
    }
}
