use std::sync::Arc;

use anyhow::{Context, Result};
use iroh::endpoint::{RecvStream, SendStream};

use super::connection::PeerConnection;
use super::messages::{SyncMessage, SyncStatus};
use crate::db::Database;

/// Handles a bi-directional stream from a peer.
pub async fn handle_stream(
    db: Database,
    peer_conn: Arc<PeerConnection>,
    (mut send, mut recv): (SendStream, RecvStream),
) -> Result<()> {
    // Read the incoming message.
    let message = read_message(&mut recv).await?;

    match message {
        SyncMessage::SyncRequest {
            requester_did,
            owner_did,
            record_id,
            version,
        } => {
            handle_sync_request(
                &db,
                &peer_conn,
                &mut send,
                requester_did,
                owner_did,
                record_id,
                version,
            )
            .await?;
        }
        SyncMessage::UpdatePush {
            record_id,
            owner_did,
            ops,
            from_version,
            signature,
        } => {
            handle_update_push(&db, &record_id, &owner_did, ops, from_version, signature).await?;

            // Send acknowledgment.
            let ack = SyncMessage::UpdateAck {
                record_id,
                owner_did,
            };
            write_message(&mut send, &ack).await?;
        }
        SyncMessage::SyncResponse { .. } | SyncMessage::UpdateAck { .. } => {
            // These are responses, not requests - unexpected on server side.
            tracing::warn!("received unexpected response message");
        }
    }

    send.finish()?;
    Ok(())
}

/// Handles a sync request from a peer.
async fn handle_sync_request(
    _db: &Database,
    peer_conn: &Arc<PeerConnection>,
    send: &mut SendStream,
    requester_did: String,
    owner_did: String,
    record_id: String,
    _version: Vec<u8>,
) -> Result<()> {
    // TODO: Verify requester identity.
    // For now, accept all requests (stub).
    let _requester_verified = verify_requester_identity(&requester_did).await;

    // TODO: Check access permission.
    // For now, allow all access (stub).
    let _access_allowed = check_access_permission(&requester_did, &owner_did, &record_id).await;

    // Register this sync in the peer connection.
    peer_conn.register_sync(owner_did.clone(), record_id.clone());

    // TODO: Load record and compare versions.
    // For now, just respond with NotFound since we don't have record loading here.
    // The actual implementation will need DataStore access.
    let response = SyncMessage::SyncResponse {
        status: SyncStatus::NotFound,
        ops: Vec::new(),
        signature: None,
    };

    write_message(send, &response).await?;

    Ok(())
}

/// Handles an update push from a peer.
async fn handle_update_push(
    _db: &Database,
    _record_id: &str,
    _owner_did: &str,
    _ops: Vec<u8>,
    _from_version: Vec<u8>,
    _signature: super::messages::SignatureWire,
) -> Result<()> {
    // TODO: Verify signature and apply update.
    // For now, just log and accept (stub).
    tracing::debug!("received update push (not yet implemented)");
    Ok(())
}

/// Verifies the identity of a requester.
///
/// TODO: Implement proper DID verification.
#[allow(clippy::unused_async)]
async fn verify_requester_identity(_requester_did: &str) -> bool {
    // Stub: always return true for now.
    true
}

/// Checks if a requester has permission to sync a record.
///
/// TODO: Implement proper access control.
#[allow(clippy::unused_async)]
async fn check_access_permission(_requester_did: &str, _owner_did: &str, _record_id: &str) -> bool {
    // Stub: always return true for now.
    true
}

/// Reads a message from a receive stream.
async fn read_message(recv: &mut RecvStream) -> Result<SyncMessage> {
    // Read length prefix (4 bytes, big-endian).
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf)
        .await
        .context("read message length")?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // Read message body.
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf)
        .await
        .context("read message body")?;

    // Decode message.
    let (message, _) =
        bincode::decode_from_slice(&buf, bincode::config::standard()).context("decode message")?;

    Ok(message)
}

/// Writes a message to a send stream.
async fn write_message(send: &mut SendStream, message: &SyncMessage) -> Result<()> {
    // Encode message.
    let buf =
        bincode::encode_to_vec(message, bincode::config::standard()).context("encode message")?;

    // Write length prefix (4 bytes, big-endian).
    let len = u32::try_from(buf.len()).context("message too large")?;
    send.write_all(&len.to_be_bytes())
        .await
        .context("write message length")?;

    // Write message body.
    send.write_all(&buf).await.context("write message body")?;

    Ok(())
}
