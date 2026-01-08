use bevy::log::error;
use iroh::protocol::ProtocolHandler;

mod inbound;
pub mod outbound;

pub const ALPN: &[u8] = b"wired/space";

#[derive(Debug)]
pub struct SpaceProtocol;

impl ProtocolHandler for SpaceProtocol {
    async fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> Result<(), iroh::protocol::AcceptError> {
        let (_tx, rx) = connection.accept_bi().await?;

        if let Err(err) = inbound::handle_inbound(rx).await {
            error!(?err, "error handling space protocol");
        }

        Ok(())
    }
}
