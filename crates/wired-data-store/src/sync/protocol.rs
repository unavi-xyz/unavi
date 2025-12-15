use std::{fmt::Debug, future::Future, sync::Arc};

use iroh::{
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};

use super::{connection::ConnectionPool, session::SyncSession};
use crate::db::Database;

/// ALPN protocol identifier for wired-sync.
pub const ALPN: &[u8] = b"wired-sync/0";

/// Sync protocol handler for incoming connections.
#[derive(Clone)]
pub struct WiredSyncProtocol {
    db: Database,
    pool: Arc<ConnectionPool>,
}

impl Debug for WiredSyncProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("WiredSyncProtocol"))
    }
}

impl WiredSyncProtocol {
    /// Creates a new sync protocol handler.
    #[must_use]
    pub const fn new(db: Database, pool: Arc<ConnectionPool>) -> Self {
        Self { db, pool }
    }
}

impl ProtocolHandler for WiredSyncProtocol {
    fn accept(
        &self,
        connection: Connection,
    ) -> impl Future<Output = Result<(), AcceptError>> + Send {
        let db = self.db.clone();
        let pool = Arc::clone(&self.pool);

        async move {
            let peer_id = connection.remote_id();

            // Register this incoming connection.
            let peer_conn = pool.register_incoming(peer_id, connection.clone()).await;

            // Accept bi-directional streams in a loop.
            loop {
                let stream = match connection.accept_bi().await {
                    Ok((send, recv)) => (send, recv),
                    Err(e) => {
                        tracing::debug!("connection closed: {e}");
                        break;
                    }
                };

                let db = db.clone();
                let peer_conn = Arc::clone(&peer_conn);

                tokio::spawn(async move {
                    let (send, recv) = stream;
                    let session = SyncSession::new(db, peer_conn, send, recv);
                    if let Err(e) = session.run().await {
                        tracing::warn!("sync session error: {e}");
                    }
                });
            }

            // Clean up connection from pool.
            pool.remove(&peer_id).await;

            Ok(())
        }
    }
}
