use iroh::{
    endpoint::{
        Connection,
        VarInt,
    },
    protocol::{
        AcceptError,
        ProtocolHandler,
    },
};
use tracing::error;

use crate::{
    connection::{
        claim_connection,
        release_connection,
    },
    peer::self_peer_id,
};

#[derive(Debug)]
pub struct SpaceProtocol;

impl ProtocolHandler for SpaceProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let peer = connection.remote_id();

        // The remote dialed, so this connection is canonical only if their id is
        // greater than ours.
        let canonical = self_peer_id().is_none_or(|s| *peer.as_bytes() > s);
        let Some((token, cancel_rx)) = claim_connection(peer, canonical) else {
            connection.close(VarInt::from_u32(1), b"already connected");
            return Ok(());
        };

        if let Err(err) = super::shared::handle_connection(connection, cancel_rx).await {
            error!(?err);
            // On error disconnect, it is up to the "client" side to re-connect.
        }

        // The peer's replicated state is owned by its inbound stream's
        // `RemotePeer` entity, despawned when the stream ends.
        release_connection(peer, token);

        Ok(())
    }
}
