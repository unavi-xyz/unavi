use anyhow::Result;

use super::messages::SignatureWire;
use crate::db::Database;

/// Handles an incoming update push from a peer.
///
/// TODO: Implement proper signature verification and ops application.
#[allow(clippy::unused_async)]
pub async fn handle_update_push(
    _db: &Database,
    _record_id: &str,
    _owner_did: &str,
    _ops: &[u8],
    _from_version: &[u8],
    _signature: &SignatureWire,
) -> Result<()> {
    // Stub: log and accept for now.
    tracing::debug!("received update push (not yet implemented)");
    Ok(())
}

/// Verifies a signature over update ops.
///
/// TODO: Implement proper signature verification.
#[allow(dead_code, clippy::unused_async)]
pub async fn verify_signature(
    _ops: &[u8],
    _signature: &SignatureWire,
    _owner_did: &str,
) -> Result<bool> {
    // Stub: always trust for now.
    Ok(true)
}
