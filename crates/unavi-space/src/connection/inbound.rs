use std::sync::Arc;

use iroh::{
    endpoint::{Connection, VarInt},
    protocol::{AcceptError, ProtocolHandler},
};
use tokio::sync::Notify;
use tracing::error;

use crate::connection::CONNECTIONS;

#[derive(Debug)]
pub struct SpaceProtocol;

impl ProtocolHandler for SpaceProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let peer = connection.remote_id();

        let cancel = {
            let mut conns = CONNECTIONS.lock().await;
            if conns.contains_key(&peer) {
                connection.close(VarInt::from_u32(1), b"already connected");
                return Ok(());
            }

            let cancel = Arc::new(Notify::default());
            conns.insert(peer, Arc::clone(&cancel));

            cancel
        };

        if let Err(err) = super::shared::handle_connection(connection, &cancel).await {
            error!(?err);
            // On disconnect, it is up to the "client" side to re-connect.
        }

        let mut conns = CONNECTIONS.lock().await;
        conns.remove(&peer);
        drop(conns);

        Ok(())
    }
}
