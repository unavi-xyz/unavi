use std::sync::Arc;

use derive_more::Debug;
use iroh::{Endpoint, EndpointId, protocol::Router};
use iroh_blobs::api::Store as BlobStore;
use irpc::Client;
use n0_future::task::AbortOnDropHandle;
use parking_lot::RwLock;
use xdid::core::did::Did;

pub use identity::Identity;

use crate::builder::{BoxedBlobs, DataStoreBuilder};

pub mod actor;
pub mod api;
mod auth;
pub mod builder;
pub mod db;
pub mod error;
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
