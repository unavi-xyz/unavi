//! Agent inbound streaming - receives agent data from remote peers.
//!
//! Stream routing is handled by `SpaceProtocol` in mod.rs.
//! Datagram handling is in `datagram.rs`.

use std::sync::atomic::Ordering;

use anyhow::bail;
use bevy::log::info;
use iroh::EndpointId;

use crate::networking::thread::{
    InboundState,
    space::{
        MAX_AGENT_TICKRATE,
        buffer::CONTROL_MSG_MAX_SIZE,
        msg::{AgentIFrameMsg, ControlMsg},
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
            bail!("I-frame message too large: {len}");
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

/// Handle tickrate negotiation request (responder side).
pub async fn handle_tickrate_request(
    mut tx: iroh::endpoint::SendStream,
    mut rx: iroh::endpoint::RecvStream,
    state: &InboundState,
) -> anyhow::Result<()> {
    let mut len_buf = [0u8; 4];
    rx.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;

    if len > CONTROL_MSG_MAX_SIZE {
        bail!("control message too large: {len}");
    }

    let mut recv_buf = [0u8; CONTROL_MSG_MAX_SIZE];
    rx.read_exact(&mut recv_buf[..len]).await?;

    let request: ControlMsg = postcard::from_bytes(&recv_buf[..len])?;
    let ControlMsg::TickrateRequest { hz: their_hz } = request else {
        bail!("expected TickrateRequest, got {request:?}");
    };

    let effective = their_hz.min(MAX_AGENT_TICKRATE);
    state.tickrate.store(effective, Ordering::Relaxed);

    let ack = ControlMsg::TickrateAck { hz: effective };
    let mut send_buf = [0u8; CONTROL_MSG_MAX_SIZE];
    let bytes = postcard::to_slice(&ack, &mut send_buf)?;
    let len = u32::try_from(bytes.len())?;
    tx.write_all(&len.to_le_bytes()).await?;
    tx.write_all(bytes).await?;

    info!(hz = effective, "agent tickrate negotiated (inbound)");

    loop {
        let mut len_buf = [0u8; 4];
        if rx.read_exact(&mut len_buf).await.is_err() {
            break;
        }
    }

    Ok(())
}
