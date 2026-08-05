use std::sync::Arc;

use derive_more::Debug;
pub use identity::Identity;
use iroh::{
    Endpoint,
    EndpointAddr,
    EndpointId,
};
use iroh_blobs::api::Store as BlobStore;
use iroh_docs::protocol::Docs;
use iroh_gossip::Gossip;
use irpc::Client;
use n0_future::task::AbortOnDropHandle;
use parking_lot::RwLock;
use xdid::core::did::Did;

use crate::builder::{
    BoxedBlobs,
    DataStoreBuilder,
};

/// DID document service `type` value identifying a WDS endpoint.
pub const WDS_SERVICE_TYPE: &str = "WDSEndpoint";

pub mod actor;
mod auth;
pub mod builder;
pub mod control;
pub mod db;
pub mod docs;
pub mod entries;
pub mod error;
mod gc;
pub mod identity;
pub mod kv;
mod quota;
pub mod signed_bytes;
pub mod tag;

pub struct DataStore {
    control_client: Client<control::ControlService>,
    auth_client:    Client<auth::AuthService>,
    endpoint:       Endpoint,
    ctx:            Arc<StoreContext>,
    _gc_handle:     Option<AbortOnDropHandle<()>>,
}

// TODO: Replace session token auth with iroh hooks
pub type SessionToken = [u8; 32];

#[derive(Debug)]
struct StoreContext {
    #[debug("BlobStore")]
    blobs:         BoxedBlobs,
    #[debug("HashMap({})", connections.len())]
    connections:   scc::HashMap<SessionToken, ConnectionState>,
    #[debug("Database")]
    db:            db::Database,
    #[debug("Docs")]
    docs:          Docs,
    #[debug("Endpoint")]
    endpoint:      Endpoint,
    #[debug("Gossip")]
    gossip:        Gossip,
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
    pub const fn builder(endpoint: Endpoint) -> DataStoreBuilder {
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
            self.endpoint.addr(),
            self.control_client.clone(),
            self.auth_client.clone(),
        )
    }

    /// Create an actor targeting a remote WDS.
    #[must_use]
    pub fn remote_actor(&self, identity: Arc<Identity>, host: EndpointAddr) -> actor::Actor {
        let control_client = irpc_iroh::client(self.endpoint.clone(), host.clone(), control::ALPN);
        let auth_client = irpc_iroh::client(self.endpoint.clone(), host.clone(), auth::ALPN);
        actor::Actor::new(identity, host, control_client, auth_client)
    }

    /// Returns the docs protocol handle. Primarily for local doc access.
    #[must_use]
    pub fn docs(&self) -> &Docs {
        &self.ctx.docs
    }

    /// The one gossip instance for this endpoint.
    ///
    /// `iroh_gossip::ALPN` can be accepted only once per router, so a second
    /// instance registering it takes every inbound connection from the first,
    /// leaving that one able to dial out and never to receive. Anything that
    /// wants a gossip topic subscribes on this.
    #[must_use]
    pub fn gossip(&self) -> &Gossip {
        &self.ctx.gossip
    }

    /// Resolves an authenticated session token to the DID that holds it.
    ///
    /// Lets another service co-deployed on this node reuse the sessions this
    /// store has already established, rather than running a second handshake.
    pub async fn session_did(&self, token: &SessionToken) -> Option<Did> {
        self.ctx
            .connections
            .read_async(token, |_, c| c.did.clone())
            .await
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
    pub fn set_user_identity(&self, identity: Arc<Identity>) {
        *self.ctx.user_identity.write() = Some(identity);
    }

    /// Runs garbage collection on the data store.
    pub async fn run_gc(&self) -> anyhow::Result<()> {
        self.ctx.run_gc().await
    }

    /// Returns the database. Primarily for testing.
    #[must_use]
    pub fn db(&self) -> &db::Database {
        &self.ctx.db
    }
}
