use std::collections::HashSet;
use std::sync::Arc;

use iroh::endpoint::Connection;
use iroh::{Endpoint, EndpointAddr, EndpointId};
use parking_lot::RwLock;
use scc::HashMap as SccHashMap;

use super::protocol::ALPN;

/// Manages active connections to sync peers.
///
/// Ensures at most ONE connection per peer [`EndpointId`], shared across all
/// records and users.
#[derive(Debug, Default)]
pub struct ConnectionPool {
    connections: SccHashMap<EndpointId, Arc<PeerConnection>>,
}

/// Active connection to a peer.
#[derive(Debug)]
pub struct PeerConnection {
    connection: Connection,
    /// Records being synced on this connection: (`owner_did`, `record_id`).
    active_syncs: RwLock<HashSet<(String, String)>>,
}

impl PeerConnection {
    /// Creates a new peer connection.
    fn new(connection: Connection) -> Self {
        Self {
            connection,
            active_syncs: RwLock::new(HashSet::new()),
        }
    }

    /// Returns the underlying iroh connection.
    pub const fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Registers a record as being actively synced.
    pub fn register_sync(&self, owner_did: String, record_id: String) {
        self.active_syncs.write().insert((owner_did, record_id));
    }

    /// Unregisters a record from active sync.
    pub fn unregister_sync(&self, owner_did: &str, record_id: &str) {
        self.active_syncs
            .write()
            .remove(&(owner_did.to_string(), record_id.to_string()));
    }

    /// Returns whether any records are being synced.
    pub fn has_active_syncs(&self) -> bool {
        !self.active_syncs.read().is_empty()
    }
}

impl ConnectionPool {
    /// Creates a new empty connection pool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets an existing connection or establishes a new one.
    ///
    /// # Errors
    ///
    /// Returns error if connection cannot be established.
    pub async fn get_or_connect(
        &self,
        endpoint: &Endpoint,
        peer_addr: impl Into<EndpointAddr>,
    ) -> anyhow::Result<Arc<PeerConnection>> {
        let peer_addr = peer_addr.into();
        let peer_id = peer_addr.id;

        // Check for existing connection.
        if let Some(entry) = self.connections.get_async(&peer_id).await {
            return Ok(entry.get().clone());
        }

        // Establish new connection.
        let connection = endpoint.connect(peer_addr, ALPN).await?;

        let peer_conn = Arc::new(PeerConnection::new(connection));
        let _ = self
            .connections
            .insert_async(peer_id, peer_conn.clone())
            .await;

        Ok(peer_conn)
    }

    /// Gets an existing connection if one exists.
    pub async fn get(&self, peer_id: &EndpointId) -> Option<Arc<PeerConnection>> {
        self.connections
            .get_async(peer_id)
            .await
            .map(|entry| entry.get().clone())
    }

    /// Registers an incoming connection from a peer.
    pub async fn register_incoming(
        &self,
        peer_id: EndpointId,
        connection: Connection,
    ) -> Arc<PeerConnection> {
        // Check if we already have a connection (race condition).
        if let Some(entry) = self.connections.get_async(&peer_id).await {
            return entry.get().clone();
        }

        let peer_conn = Arc::new(PeerConnection::new(connection));
        let _ = self
            .connections
            .insert_async(peer_id, peer_conn.clone())
            .await;

        peer_conn
    }

    /// Removes a connection from the pool.
    pub async fn remove(&self, peer_id: &EndpointId) {
        let _ = self.connections.remove_async(peer_id).await;
    }

    /// Returns the number of active connections.
    pub fn len(&self) -> usize {
        self.connections.len()
    }

    /// Returns whether the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }
}
