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

use crate::connection::PeerLink;

pub struct SpaceProtocol {
    link: PeerLink,
}

impl SpaceProtocol {
    pub const fn new(link: PeerLink) -> Self {
        Self { link }
    }
}

impl std::fmt::Debug for SpaceProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpaceProtocol").finish_non_exhaustive()
    }
}

impl ProtocolHandler for SpaceProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let peer = connection.remote_id();

        // A block is retroactive, so it has to refuse the next connection as
        // well as undo the last one.
        if self.link.is_blocked(peer) {
            connection.close(VarInt::from_u32(2), b"blocked");
            return Ok(());
        }

        // The remote dialed, so this connection is canonical only if their id is
        // greater than ours.
        let canonical = peer > self.link.view().me();
        let Some((token, cancel_rx)) = self.link.claim_connection(peer, canonical) else {
            connection.close(VarInt::from_u32(1), b"already connected");
            return Ok(());
        };

        if let Err(err) = super::shared::handle_connection(&self.link, connection, cancel_rx).await
        {
            error!(?err);
            // On error disconnect, it is up to the "client" side to re-connect.
        }

        // The peer's replicated state is owned by its inbound stream's
        // `RemotePeer` entity, despawned when the stream ends.
        self.link.release_connection(peer, token);

        Ok(())
    }
}
