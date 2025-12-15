use std::sync::Arc;

use anyhow::{Context, Result};
use iroh::endpoint::{RecvStream, SendStream};

use super::{
    connection::PeerConnection,
    messages::{SyncMessage, SyncStatus},
    request::{authenticate, check_access},
    update::handle_update_push,
    wire::{read_message, write_message},
};
use crate::db::Database;

/// Manages a single bidirectional sync session for one record.
///
/// Each session corresponds to one iroh bi-stream and handles:
/// 1. Initial handshake (SyncRequest/SyncResponse)
/// 2. Bidirectional version exchange and catchup
/// 3. Live update propagation until stream closes
pub struct SyncSession {
    db: Database,
    peer_conn: Arc<PeerConnection>,
    send: SendStream,
    recv: RecvStream,
}

impl SyncSession {
    /// Creates a new sync session from an accepted bi-stream.
    pub const fn new(
        db: Database,
        peer_conn: Arc<PeerConnection>,
        send: SendStream,
        recv: RecvStream,
    ) -> Self {
        Self {
            db,
            peer_conn,
            send,
            recv,
        }
    }

    /// Runs the sync session to completion.
    ///
    /// This is the main entry point, called after the stream is accepted.
    /// Returns when the stream closes or an error occurs.
    pub async fn run(mut self) -> Result<()> {
        // Phase 1: Receive and validate initial request.
        let Some((owner_did, record_id)) = self.handle_request().await? else {
            return Ok(()); // Auth failed, response already sent.
        };

        // Phase 2: Initial sync (exchange versions, catchup ops).
        self.initial_sync(&owner_did, &record_id).await?;

        // Phase 3: Live sync loop.
        self.live_sync_loop(&owner_did, &record_id).await?;

        // Clean up.
        self.send.finish()?;
        Ok(())
    }

    /// Handles the initial `SyncRequest` and authentication.
    ///
    /// Returns `Some((owner_did, record_id))` on success, `None` if auth failed.
    async fn handle_request(&mut self) -> Result<Option<(String, String)>> {
        // Read the opening message (must be SyncRequest).
        let message = read_message(&mut self.recv).await?;

        let SyncMessage::SyncRequest {
            requester_did,
            owner_did,
            record_id,
            version: _version,
        } = message
        else {
            tracing::warn!("session opened with non-SyncRequest message");
            return Ok(None);
        };

        // Authenticate requester.
        let auth_result = authenticate(&requester_did).await;
        if !auth_result.ok {
            let response = SyncMessage::SyncResponse {
                status: SyncStatus::Unauthorized,
                ops: Vec::new(),
                signature: None,
            };
            write_message(&mut self.send, &response).await?;
            return Ok(None);
        }

        // Check access permission.
        let has_access = check_access(&requester_did, &owner_did, &record_id).await;
        if !has_access {
            let response = SyncMessage::SyncResponse {
                status: SyncStatus::AccessDenied,
                ops: Vec::new(),
                signature: None,
            };
            write_message(&mut self.send, &response).await?;
            return Ok(None);
        }

        // Register this sync session.
        self.peer_conn
            .register_sync(owner_did.clone(), record_id.clone());

        Ok(Some((owner_did, record_id)))
    }

    /// Performs initial sync: exchange versions and send catchup ops.
    async fn initial_sync(&mut self, _owner_did: &str, _record_id: &str) -> Result<()> {
        // TODO: Load record, compare versions, export catchup ops.
        // For now, respond with NotFound since we don't have record loading.

        let response = SyncMessage::SyncResponse {
            status: SyncStatus::NotFound,
            ops: Vec::new(),
            signature: None,
        };
        write_message(&mut self.send, &response).await?;

        // TODO: Receive requester's catchup ops if they're ahead.

        Ok(())
    }

    /// Runs the live sync loop, handling updates until stream closes.
    async fn live_sync_loop(&mut self, owner_did: &str, record_id: &str) -> Result<()> {
        loop {
            // Read next message.
            let message = match read_message(&mut self.recv).await {
                Ok(msg) => msg,
                Err(e) => {
                    // Stream closed or error - exit gracefully.
                    tracing::debug!("sync session ended: {e}");
                    break;
                }
            };

            match message {
                SyncMessage::UpdatePush {
                    record_id: push_record_id,
                    owner_did: push_owner_did,
                    ops,
                    from_version,
                    signature,
                } => {
                    // Verify this update is for the session's record.
                    if push_record_id != record_id || push_owner_did != owner_did {
                        tracing::warn!(
                            "received update for wrong record: expected {owner_did}/{record_id}"
                        );
                        continue;
                    }

                    // Process the update.
                    handle_update_push(
                        &self.db,
                        &push_record_id,
                        &push_owner_did,
                        &ops,
                        &from_version,
                        &signature,
                    )
                    .await
                    .context("handle update push")?;

                    // Send acknowledgment.
                    let ack = SyncMessage::UpdateAck {
                        record_id: push_record_id,
                        owner_did: push_owner_did,
                    };
                    write_message(&mut self.send, &ack).await?;
                }
                SyncMessage::UpdateAck { .. } => {
                    // Acknowledgment received - nothing to do for now.
                    // TODO: Track which updates have been acked.
                }
                SyncMessage::SyncRequest { .. } => {
                    tracing::warn!("unexpected SyncRequest during live sync");
                }
                SyncMessage::SyncResponse { .. } => {
                    tracing::warn!("unexpected SyncResponse during live sync");
                }
            }
        }

        // Unregister sync on exit.
        self.peer_conn.unregister_sync(owner_did, record_id);

        Ok(())
    }
}
