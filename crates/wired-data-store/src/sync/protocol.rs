use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;

use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};

use super::connection::ConnectionPool;
use super::handler::handle_stream;
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
        let pool = self.pool.clone();

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
                let peer_conn = peer_conn.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_stream(db, peer_conn, stream).await {
                        tracing::warn!("stream error: {e}");
                    }
                });
            }

            // Clean up connection from pool.
            pool.remove(&peer_id).await;

            Ok(())
        }
    }
}
