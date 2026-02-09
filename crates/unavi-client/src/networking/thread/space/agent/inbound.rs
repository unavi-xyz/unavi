//! Agent inbound streaming - receives agent data from remote peers.
//!
//! Stream routing is handled by `SpaceProtocol` in mod.rs.
//! Datagram handling is in `datagram.rs`.

use std::sync::atomic::Ordering;

use bevy::log::debug;
use iroh::EndpointId;

use crate::networking::thread::{
    InboundState,
    space::{
        MAX_AGENT_TICKRATE,
        buffer::CONTROL_MSG_MAX_SIZE,
        msg::{AgentIFrameMsg, ControlMsg, read_control, write_control},
    },
};

/// Receive agent I-frames from the reliable stream.
pub async fn recv_agent_iframes(
    mut stream: iroh::endpoint::RecvStream,
    state: &InboundState,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] remote: EndpointId,
) -> anyhow::Result<()> {
    loop {
        // Read length prefix.
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).await.is_err() {
            // Stream closed.
            break;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        if len > 64 * 1024 {
            anyhow::bail!("I-frame message too large: {len}");
        }

        // Read message.
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;

        let msg: AgentIFrameMsg = postcard::from_bytes(&buf)?;

        #[cfg(feature = "devtools-network")]
        {
            use crate::devtools::events::{Channel, NETWORK_EVENTS, NetworkEvent};
            let _ = NETWORK_EVENTS.0.try_send(NetworkEvent::Download {
                peer: remote,
                bytes: buf.len() + 4,
                channel: Channel::Stream,
            });
            let _ = NETWORK_EVENTS
                .0
                .try_send(NetworkEvent::ValidTick { peer: remote });
        }

        *state.pose.iframe.lock() = Some(msg);
    }

    Ok(())
}

/// Handle control stream (responder side) - continuous loop.
/// Receives tickrate requests and sends acks.
pub async fn handle_control_stream(
    mut tx: iroh::endpoint::SendStream,
    mut rx: iroh::endpoint::RecvStream,
    state: &InboundState,
) -> anyhow::Result<()> {
    let mut buf = [0u8; CONTROL_MSG_MAX_SIZE];

    loop {
        let msg: ControlMsg = read_control(&mut rx, &mut buf).await?;

        match msg {
            ControlMsg::TickrateRequest { hz } => {
                let effective = hz.min(MAX_AGENT_TICKRATE);
                state.tickrate.store(effective, Ordering::Relaxed);

                write_control(&mut tx, &ControlMsg::TickrateAck { hz: effective }, &mut buf).await?;
                debug!(hz = effective, "agent tickrate updated (inbound)");
            }
            ControlMsg::TickrateAck { hz } => {
                state.tickrate.store(hz, Ordering::Relaxed);
            }
        }
    }
}
