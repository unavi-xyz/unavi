use iroh::{
    endpoint::{Connection, VarInt},
    protocol::{AcceptError, ProtocolHandler},
};
use tokio::sync::oneshot;
use tracing::error;

use crate::connection::CONNECTIONS;

#[derive(Debug)]
pub struct SpaceProtocol;

impl ProtocolHandler for SpaceProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let peer = connection.remote_id();

        let cancel_rx = {
            let mut conns = CONNECTIONS.lock().expect("connections lock");
            if conns.contains_key(&peer) {
                connection.close(VarInt::from_u32(1), b"already connected");
                return Ok(());
            }

            let (cancel_tx, cancel_rx) = oneshot::channel();
            conns.insert(peer, cancel_tx);

            cancel_rx
        };

        if let Err(err) = super::shared::handle_connection(connection, cancel_rx).await {
            error!(?err);
            // On error disconnect, it is up to the "client" side to re-connect.
        }

        let mut conns = CONNECTIONS.lock().expect("connections lock");
        conns.remove(&peer);
        drop(conns);

        Ok(())
    }
}
