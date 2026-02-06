//! Real-time networking for agents and objects within a space.
//!
//! Each connection is unidirectional for pose data:
//! - Sender opens connection, sends I-frames (stream) and P-frames (datagrams)
//! - Receiver accepts, stores latest frames for Bevy to read
//!
//! Tickrate negotiation uses a bidirectional control stream within each connection.

use std::sync::Arc;

use bevy::log::error;
use iroh::{EndpointId, protocol::ProtocolHandler};
use iroh_gossip::api::GossipSender;

use crate::networking::thread::NetworkEvent;

use self::ownership::ObjectOwnership;
use super::InboundState;

pub mod agent;
pub mod buffer;
pub mod gossip;
pub mod msg;
pub mod object;
pub mod ownership;
pub mod types;

pub const ALPN: &[u8] = b"wired/space";
pub const MAX_TICKRATE: u8 = 20;
pub const MIN_TICKRATE: u8 = 5;

/// Per-space shared state for gossip and ownership.
pub struct SpaceHandle {
    pub gossip_tx: GossipSender,
    pub ownership: Arc<ObjectOwnership>,
}

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
        if let Err(err) = agent::inbound::handle_inbound(
            self.event_tx.clone(),
            Arc::clone(&self.inbound),
            connection,
        )
        .await
        {
            error!(?err, "error handling space protocol inbound");
        }

        Ok(())
    }
}
