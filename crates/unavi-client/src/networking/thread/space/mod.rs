//! Real-time networking for players within a space.
//!
//! Each player sets a tickrate for both inbound and outbound updates.
//! The slowest tickrate between the two is used.
//!
//! This tickrate is variable. As they move further away, a player may choose to
//! limit the updates sent or recieved to the other player, as on optimization.

use bevy::log::error;
use iroh::protocol::ProtocolHandler;

mod inbound;
pub(super) mod outbound;
mod pos;
mod pose;
mod quat;

pub use pose::{BonePose, IFrameTransform, PFrameTransform, PlayerIFrame, PlayerPFrame};

pub const ALPN: &[u8] = b"wired/space";

#[derive(Debug)]
pub struct SpaceProtocol;

impl ProtocolHandler for SpaceProtocol {
    async fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> Result<(), iroh::protocol::AcceptError> {
        let (tx, rx) = connection.accept_bi().await?;

        if let Err(err) = inbound::handle_inbound(tx, rx).await {
            error!(?err, "error handling space protocol");
        }

        Ok(())
    }
}
