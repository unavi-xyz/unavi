//! Real-time networking for players within a space.
//!
//! Each connection is unidirectional for pose data:
//! - Sender opens connection, sends I-frames (stream) and P-frames (datagrams)
//! - Receiver accepts, stores latest frames for Bevy to read
//!
//! Tickrate negotiation uses a bidirectional control stream within each connection.

use std::sync::Arc;

use bevy::log::error;
use iroh::{EndpointId, protocol::ProtocolHandler};

use crate::networking::thread::NetworkEvent;

use super::InboundState;

mod inbound;
pub mod msg;
pub(super) mod outbound;
mod pos;
mod pose;
mod quat;

pub use msg::{ControlMsg, IFrameMsg, PFrameDatagram};
pub use pose::{BonePose, IFrameTransform, PFrameTransform, PlayerIFrame, PlayerPFrame};

pub const ALPN: &[u8] = b"wired/space";
pub const DEFAULT_TICKRATE: u8 = 20;

/// Protocol handler for accepting inbound pose connections.
#[derive(Debug, Clone)]
pub struct SpaceProtocol {
    pub event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    pub inbound: Arc<scc::HashMap<EndpointId, Arc<InboundState>>>,
}

impl ProtocolHandler for SpaceProtocol {
    async fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> Result<(), iroh::protocol::AcceptError> {
        if let Err(err) =
            inbound::handle_inbound(self.event_tx.clone(), Arc::clone(&self.inbound), connection)
                .await
        {
            error!(?err, "error handling space protocol inbound");
        }

        Ok(())
    }
}
