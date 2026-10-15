//! Accepting `wired/space/1` connections.

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

use crate::link::{
    PeerLink,
    streams,
};

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

        // The remote dialed, so this connection is canonical only if their id
        // is greater than ours.
        let canonical = peer > self.link.view().me();
        let Some((slot, cancel_rx)) = self.link.claim_connection(peer, canonical) else {
            connection.close(VarInt::from_u32(1), b"already connected");
            return Ok(());
        };

        // On error the dialing side reconnects.
        if let Err(err) = streams::handle_connection(&self.link, &slot, connection, cancel_rx).await
        {
            error!(?err);
        }
        Ok(())
    }
}
