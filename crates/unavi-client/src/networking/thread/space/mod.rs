//! Real-time networking for agents and objects within a space.
//!
//! Uses a single ALPN protocol with multiplexed streams identified by init message:
//! - `StreamInit::AgentControl` - bistream for tickrate negotiation
//! - `StreamInit::AgentIFrame` - unistream for agent I-frames
//! - `StreamInit::Object { object_id }` - bistream for object I-frames
//!
//! Datagrams are shared and tagged by type (`AgentPFrame` or `ObjectPFrame`).

use std::sync::Arc;

use bevy::log::{debug, error, info, warn};
use iroh::{EndpointId, protocol::ProtocolHandler};
use iroh_gossip::api::GossipSender;

use self::{msg::StreamInit, ownership::ObjectOwnership};
use crate::networking::thread::{InboundState, NetworkEvent};

pub mod agent;
pub mod buffer;
pub mod datagram;
pub mod gossip;
pub mod msg;
pub mod object;
pub mod outbound;
pub mod ownership;
pub mod types;

/// ALPN for space protocol (agent and object sync).
pub const ALPN: &[u8] = b"wired/space";

pub const MAX_AGENT_TICKRATE: u8 = 20;
pub const MAX_OBJECT_TICKRATE: u8 = 10;
pub const MIN_TICKRATE: u8 = 5;

/// Per-space shared state for gossip and ownership.
pub struct SpaceHandle {
    pub gossip_tx: GossipSender,
    pub ownership: Arc<ObjectOwnership>,
}

/// Protocol handler for accepting inbound space connections.
/// Routes streams based on `StreamInit` message.
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
        let remote = connection.remote_id();
        info!("space protocol inbound from {remote}");

        // Create and register inbound state for agent data.
        let state = Arc::new(InboundState::default());
        if self
            .inbound
            .insert_async(remote, Arc::clone(&state))
            .await
            .is_err()
        {
            warn!("failed to insert inbound state for {remote}");
        }

        let _ = self
            .event_tx
            .send(NetworkEvent::AgentJoin {
                id: remote,
                state: Arc::clone(&state),
            })
            .await;

        // Handle the connection with stream routing.
        if let Err(err) = handle_inbound_connection(
            self.event_tx.clone(),
            Arc::clone(&self.inbound),
            connection,
            state,
        )
        .await
        {
            error!(?err, "error handling space protocol inbound");
        }

        // Cleanup on disconnect.
        let _ = self.inbound.remove_async(&remote).await;
        info!("space protocol inbound closed from {remote}");

        Ok(())
    }
}

/// Handle an inbound space connection, routing streams by `StreamInit`.
async fn handle_inbound_connection(
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    inbound_map: Arc<scc::HashMap<EndpointId, Arc<InboundState>>>,
    connection: iroh::endpoint::Connection,
    agent_state: Arc<InboundState>,
) -> anyhow::Result<()> {
    let remote = connection.remote_id();

    // Shared baselines for object P-frame decoding.
    let object_baselines: datagram::SharedObjectBaselines = Arc::default();

    // Spawn datagram handler for P-frames (agent and object).
    let conn_clone = connection.clone();
    let agent_state_clone = Arc::clone(&agent_state);
    let event_tx_clone = event_tx.clone();
    let baselines_clone = Arc::clone(&object_baselines);
    let datagram_handle = n0_future::task::spawn(async move {
        if let Err(err) = datagram::handle_datagrams(
            &conn_clone,
            &agent_state_clone,
            event_tx_clone,
            baselines_clone,
            remote,
        )
        .await
        {
            debug!(?err, "datagram handler closed");
        }
    });

    // Accept streams and route by init message.
    loop {
        tokio::select! {
            bistream = connection.accept_bi() => {
                match bistream {
                    Ok((send, mut recv)) => {
                        // Read stream init.
                        let init = match read_stream_init(&mut recv).await {
                            Ok(init) => init,
                            Err(err) => {
                                warn!(?err, "failed to read stream init");
                                continue;
                            }
                        };

                        match init {
                            StreamInit::AgentControl => {
                                let state = Arc::clone(&agent_state);
                                n0_future::task::spawn(async move {
                                    if let Err(err) =
                                        agent::inbound::handle_control_stream(send, recv, &state).await
                                    {
                                        debug!(?err, "control stream closed");
                                    }
                                });
                            }
                            StreamInit::AgentIFrame => {
                                warn!("received AgentIFrame on bistream, expected unistream");
                            }
                            StreamInit::ObjectControl { object_id } => {
                                let event_tx = event_tx.clone();
                                n0_future::task::spawn(async move {
                                    if let Err(err) = object::inbound::recv_object_control(
                                        event_tx, object_id, send, recv,
                                    )
                                    .await
                                    {
                                        debug!(?err, "object control stream closed");
                                    }
                                });
                            }
                            StreamInit::ObjectIFrame { .. } => {
                                warn!("received ObjectIFrame on bistream, expected unistream");
                            }
                        }
                    }
                    Err(err) => {
                        debug!(?err, "bistream accept failed");
                        break;
                    }
                }
            }
            unistream = connection.accept_uni() => {
                match unistream {
                    Ok(mut recv) => {
                        // Read stream init.
                        let init = match read_stream_init(&mut recv).await {
                            Ok(init) => init,
                            Err(err) => {
                                warn!(?err, "failed to read unistream init");
                                continue;
                            }
                        };

                        match init {
                            StreamInit::AgentIFrame => {
                                let state = Arc::clone(&agent_state);
                                n0_future::task::spawn(async move {
                                    if let Err(err) =
                                        agent::inbound::recv_agent_iframes(recv, &state, remote).await
                                    {
                                        debug!(?err, "iframe stream closed");
                                    }
                                });
                            }
                            StreamInit::AgentControl => {
                                warn!("received AgentControl on unistream, expected bistream");
                            }
                            StreamInit::ObjectControl { .. } => {
                                warn!("received ObjectControl on unistream, expected bistream");
                            }
                            StreamInit::ObjectIFrame { object_id } => {
                                let event_tx = event_tx.clone();
                                let baselines = Arc::clone(&object_baselines);
                                n0_future::task::spawn(async move {
                                    if let Err(err) = object::inbound::recv_object_iframes(
                                        event_tx, remote, object_id, recv, baselines,
                                    )
                                    .await
                                    {
                                        debug!(?err, "object iframe stream closed");
                                    }
                                });
                            }
                        }
                    }
                    Err(err) => {
                        debug!(?err, "unistream accept failed");
                        break;
                    }
                }
            }
        }
    }

    datagram_handle.abort();
    let _ = inbound_map.remove_async(&remote).await;

    Ok(())
}

/// Read and deserialize a `StreamInit` message from a stream.
async fn read_stream_init(recv: &mut iroh::endpoint::RecvStream) -> anyhow::Result<StreamInit> {
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;

    if len > 256 {
        anyhow::bail!("stream init too large: {len}");
    }

    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await?;

    Ok(postcard::from_bytes(&buf)?)
}
