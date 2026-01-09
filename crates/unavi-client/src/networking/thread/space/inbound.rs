//! Inbound connection handler - receives poses from a remote player.

use std::sync::Arc;

use anyhow::bail;
use bevy::log::{debug, info, warn};
use iroh::{EndpointId, endpoint::Connection};

use super::{ControlMsg, IFrameMsg, PFrameDatagram};
use crate::networking::thread::InboundState;

const DEFAULT_TICKRATE: u8 = 20;

pub async fn handle_inbound(
    inbound_map: Arc<scc::HashMap<EndpointId, Arc<InboundState>>>,
    connection: Connection,
) -> anyhow::Result<()> {
    let remote = connection.remote_id();
    info!("inbound connection from {remote}");

    // Accept control bistream (opened by outbound side).
    let (ctrl_tx, ctrl_rx) = connection.accept_bi().await?;

    // Accept I-frame stream (opened by outbound side).
    let iframe_stream = connection.accept_uni().await?;

    // Create and register inbound state.
    let state = Arc::new(InboundState::default());
    let _ = inbound_map.insert_async(remote, Arc::clone(&state)).await;

    // Run receive tasks.
    let result = tokio::select! {
        r = recv_iframes(iframe_stream, &state) => r,
        r = recv_pframes(&connection, &state) => r,
        r = respond_tickrate(ctrl_tx, ctrl_rx) => r,
    };

    // Cleanup on disconnect.
    let _ = inbound_map.remove_async(&remote).await;
    info!("inbound connection closed: {remote}");

    result
}

/// Receives I-frames from the reliable stream.
async fn recv_iframes(
    mut stream: iroh::endpoint::RecvStream,
    state: &InboundState,
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

        let msg: IFrameMsg = postcard::from_bytes(&buf)?;
        debug!(id = msg.id, "received I-frame");

        *state.latest_iframe.lock() = Some(msg);
    }

    Ok(())
}

/// Receives P-frames from datagrams (unreliable).
async fn recv_pframes(conn: &Connection, state: &InboundState) -> anyhow::Result<()> {
    loop {
        let datagram = conn.read_datagram().await?;
        let msg: PFrameDatagram = match postcard::from_bytes(&datagram) {
            Ok(msg) => msg,
            Err(err) => {
                warn!(?err, "invalid P-frame datagram");
                continue;
            }
        };

        // Check P-frame ID matches latest I-frame.
        let current_iframe_id = state.latest_iframe.lock().as_ref().map_or(0, |f| f.id);

        if msg.iframe_id != current_iframe_id {
            // Stale P-frame, drop it.
            debug!(
                pframe_id = msg.iframe_id,
                current_iframe_id, "dropping stale P-frame"
            );
            continue;
        }

        *state.latest_pframe.lock() = Some(msg.pose);
    }
}

/// Responds to tickrate negotiation requests.
async fn respond_tickrate(
    mut tx: iroh::endpoint::SendStream,
    mut rx: iroh::endpoint::RecvStream,
) -> anyhow::Result<()> {
    // Read their tickrate request.
    let mut len_buf = [0u8; 4];
    rx.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;

    if len > 1024 {
        bail!("control message too large: {len}");
    }

    let mut buf = vec![0u8; len];
    rx.read_exact(&mut buf).await?;

    let request: ControlMsg = postcard::from_bytes(&buf)?;
    let ControlMsg::TickrateRequest { hz: their_hz } = request else {
        bail!("expected TickrateRequest, got {request:?}");
    };

    // Respond with effective tickrate (minimum of theirs and ours).
    let effective = their_hz.min(DEFAULT_TICKRATE);
    let ack = ControlMsg::TickrateAck { hz: effective };
    let bytes = postcard::to_stdvec(&ack)?;
    let len = u32::try_from(bytes.len())?;
    tx.write_all(&len.to_le_bytes()).await?;
    tx.write_all(&bytes).await?;

    info!(hz = effective, "tickrate negotiated (inbound)");

    // Keep alive, wait for remote to close.
    loop {
        let mut len_buf = [0u8; 4];
        if rx.read_exact(&mut len_buf).await.is_err() {
            break;
        }
    }

    Ok(())
}
