use std::{
    sync::Arc,
    time::Duration,
};

use identity::{
    Identity,
    WdsIdentity,
};
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
use xdid::core::did::Did;

use crate::builder::{
    BoxedBlobs,
    DataStoreBuilder,
};

pub mod actor;
mod auth;
pub mod builder;
pub mod cache;
pub mod control;
pub mod db;
pub mod docs;
pub mod entries;
pub mod error;
mod gc;
pub mod identity;
pub mod kv;
mod quota;
pub mod resolve;
pub mod signed_bytes;
pub mod tag;

// TODO: Replace session token auth with iroh hooks
pub type SessionToken = [u8; 32];

pub const SESSION_TTL: Duration = Duration::from_hours(12);
/// DID document service `type` value identifying a WDS endpoint.
pub const WDS_SERVICE_TYPE: &str = "WDSEndpoint";

pub struct DataStore {
    control_client: Client<control::ControlService>,
    auth_client:    Client<auth::AuthService>,
    endpoint:       Endpoint,
    ctx:            Arc<StoreContext>,
    _gc_handle:     Option<AbortOnDropHandle<()>>,
}

struct StoreContext {
    blobs:       BoxedBlobs,
    connections: scc::HashMap<SessionToken, ConnectionState>,
    db:          db::Database,
    docs:        Docs,
    endpoint:    Endpoint,
    gossip:      Gossip,
    /// Namespaces this node replicates on someone's behalf, each holding the
    /// doc handle and metering task that hosting it entails.
    hosted:      scc::HashMap<iroh_docs::NamespaceId, HostedDoc>,
    identity:    Arc<WdsIdentity>,
}

struct HostedDoc {
    doc:    iroh_docs::api::Doc,
    _meter: AbortOnDropHandle<()>,
}

impl StoreContext {
    fn blob_store(&self) -> &BlobStore {
        self.blobs.as_ref().as_ref()
    }
}

struct ConnectionState {
    /// The authenticated DID of the connection.
    /// Set by the `wds/auth` protocol.
    did:     Did,
    /// Unix timestamp past which the token no longer authenticates; prevents
    /// the session table growing without bound.
    expires: i64,
}

impl DataStore {
    /// Create a new [`DataStoreBuilder`].
    #[must_use]
    pub const fn builder(endpoint: Endpoint, identity: Arc<WdsIdentity>) -> DataStoreBuilder {
        DataStoreBuilder::new(endpoint, identity)
    }

    #[must_use]
    pub fn endpoint_id(&self) -> EndpointId {
        self.ctx.endpoint.id()
    }

    /// Create an actor targeting the local WDS, acting as this node.
    #[must_use]
    pub fn local_actor(&self) -> actor::Actor {
        self.local_actor_as(Arc::clone(self.ctx.identity.user()))
    }

    /// Create an actor targeting the local WDS on behalf of some other
    /// identity.
    ///
    /// A host serves many DIDs, so its own identity is not the only one that
    /// can act against it.
    #[must_use]
    pub fn local_actor_as(&self, identity: Arc<Identity>) -> actor::Actor {
        actor::Actor::new(
            identity,
            self.endpoint.addr(),
            self.control_client.clone(),
            self.auth_client.clone(),
        )
    }

    /// Create an actor targeting a remote WDS, acting as this node.
    #[must_use]
    pub fn remote_actor(&self, host: EndpointAddr) -> actor::Actor {
        let control_client = irpc_iroh::client(self.endpoint.clone(), host.clone(), control::ALPN);
        let auth_client = irpc_iroh::client(self.endpoint.clone(), host.clone(), auth::ALPN);
        let identity = Arc::clone(self.ctx.identity.user());
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
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        self.ctx
            .connections
            .read_async(token, |_, c| (c.did.clone(), c.expires))
            .await
            .and_then(|(did, expires)| (expires > now).then_some(did))
    }

    /// Returns the blob store. Primarily for testing.
    #[must_use]
    pub fn blobs(&self) -> &BlobStore {
        self.ctx.blob_store()
    }

    /// Returns the iroh endpoint. Primarily for testing.
    #[must_use]
    pub fn endpoint(&self) -> &Endpoint {
        &self.ctx.endpoint
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
