use std::{path::Path, sync::Arc};

use iroh::{Endpoint, protocol::Router};
use iroh_blobs::{
    BlobsProtocol,
    store::fs::{FsStore, options::Options},
};
use irpc::Client;
use tokio::{sync::OnceCell, task::JoinError};
use xdid::core::did::Did;

pub mod actor;
pub mod api;
mod auth;
mod blob;
mod db;
mod quota;
pub mod signed_bytes;

/// Wired data store.
pub struct DataStore {
    api_client: Client<api::ApiService>,
    router: Router,
}

struct ConnectionState {
    /// The authenticated DID of the connection, if one existis.
    /// Set by the `wds/auth` protocol.
    authentication: OnceCell<Did>,
    blob_store: FsStore,
    db: db::Database,
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
        let blob_store = FsStore::load_with_opts(blob_db_path, Options::new(&blob_path)).await?;
        let blob_protocol = BlobsProtocol::new(&blob_store, None);

        let db_path = path.join("index.db");
        let db = db::Database::new(&db_path).await?;

        let state = Arc::new(ConnectionState {
            authentication: OnceCell::default(),
            blob_store,
            db,
        });

        let (api_client, api_protocol) = api::protocol(Arc::clone(&state));

        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, blob_protocol)
            .accept(api::ALPN, api_protocol)
            .accept(auth::ALPN, auth::protocol(state))
            .spawn();

        Ok(Self { api_client, router })
    }

    /// Local client for interacting with the data store.
    #[must_use]
    pub const fn client(&self) -> &Client<api::ApiService> {
        &self.api_client
    }

    /// # Errors
    ///
    /// Errors if protocol tasks could not be joined.
    pub async fn shutdown(self) -> Result<(), JoinError> {
        self.router.shutdown().await
    }
}
