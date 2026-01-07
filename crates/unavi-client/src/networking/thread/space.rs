use bevy::log::error;
use iroh::{endpoint::RecvStream, protocol::ProtocolHandler};

pub const ALPN: &[u8] = b"wired/space";

#[derive(Debug)]
pub struct SpaceProtocol;

impl ProtocolHandler for SpaceProtocol {
    async fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> Result<(), iroh::protocol::AcceptError> {
        let (_tx, rx) = connection.accept_bi().await?;

        if let Err(e) = handle_incoming(rx).await {
            error!(err = ?e, "error handling space protocol");
        }

        Ok(())
    }
}

async fn handle_incoming(_rx: RecvStream) -> anyhow::Result<()> {
    Ok(())
}
