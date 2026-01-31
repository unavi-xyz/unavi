//! Inbound connection handler - receives poses from a remote player.

use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use anyhow::bail;
use bevy::log::{info, warn};
use iroh::{EndpointId, endpoint::Connection};
use tracing::debug;

use super::{
    ControlMsg, DEFAULT_TICKRATE, IFrameMsg, PFrameDatagram, buffer::CONTROL_MSG_MAX_SIZE,
    reorder::PFrameReorderBuffer,
};
use crate::networking::thread::{InboundState, NetworkEvent};

pub async fn handle_inbound(
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    inbound_map: Arc<scc::HashMap<EndpointId, Arc<InboundState>>>,
    connection: Connection,
) -> anyhow::Result<()> {
    let remote = connection.remote_id();
    info!("inbound connection from {remote}");

    let (ctrl_tx, ctrl_rx) = connection.accept_bi().await?;
    let iframe_stream = connection.accept_uni().await?;

    // Create and register inbound state.
    let state = Arc::new(InboundState::default());
    inbound_map
        .insert_async(remote, Arc::clone(&state))
        .await
        .map_err(|_| anyhow::anyhow!("failed to insert inbound state"))?;

    event_tx
        .send(NetworkEvent::PlayerJoin {
            id: remote,
            state: Arc::clone(&state),
        })
        .await?;

    // Run receive tasks.
    let result = tokio::select! {
        r = recv_iframes(iframe_stream, &state, remote) => r,
        r = recv_pframes(&connection, &state, remote) => r,
        r = respond_tickrate(ctrl_tx, ctrl_rx, &state) => r,
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

        let msg: IFrameMsg = postcard::from_bytes(&buf)?;

        #[cfg(feature = "devtools-network")]
        {
            use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};
            let _ = NETWORK_EVENTS.0.try_send(NetworkEvent::Download {
                peer: remote,
                bytes: buf.len() + 4,
            });
            let _ = NETWORK_EVENTS
                .0
                .try_send(NetworkEvent::ValidTick { peer: remote });
        }

        *state.pose.iframe.lock() = Some(msg);
    }

    Ok(())
}

/// Receives P-frames from datagrams (unreliable).
async fn recv_pframes(
    conn: &Connection,
    state: &InboundState,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] remote: EndpointId,
) -> anyhow::Result<()> {
    let mut reorder = PFrameReorderBuffer::default();

    loop {
        let datagram = conn.read_datagram().await?;
        let msg: PFrameDatagram = match postcard::from_bytes(&datagram) {
            Ok(msg) => msg,
            Err(err) => {
                warn!(?err, "invalid P-frame datagram");
                continue;
            }
        };

        let current_iframe_id = state.pose.iframe.lock().as_ref().map_or(0, |f| f.id);

        if msg.iframe_id != current_iframe_id {
            #[cfg(feature = "devtools-network")]
            {
                use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};
                let _ = NETWORK_EVENTS
                    .0
                    .try_send(NetworkEvent::DroppedFrame { peer: remote });
            }

            debug!(
                pframe_id = msg.iframe_id,
                current_iframe_id, "dropping stale P-frame"
            );
            continue;
        }

        if msg.iframe_id != reorder.iframe_id() {
            reorder.reset(msg.iframe_id);
        }

        for ready_frame in reorder.insert(msg) {
            #[cfg(feature = "devtools-network")]
            {
                use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};
                let _ = NETWORK_EVENTS.0.try_send(NetworkEvent::Download {
                    peer: remote,
                    bytes: datagram.len(),
                });
                let _ = NETWORK_EVENTS
                    .0
                    .try_send(NetworkEvent::ValidTick { peer: remote });
            }

            *state.pose.pframe.lock() = Some(ready_frame);

            n0_future::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

/// Responds to tickrate negotiation requests.
async fn respond_tickrate(
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

    let effective = their_hz.min(DEFAULT_TICKRATE);
    state.tickrate.store(effective, Ordering::Relaxed);

    let ack = ControlMsg::TickrateAck { hz: effective };
    let mut send_buf = [0u8; CONTROL_MSG_MAX_SIZE];
    let bytes = postcard::to_slice(&ack, &mut send_buf)?;
    let len = u32::try_from(bytes.len())?;
    tx.write_all(&len.to_le_bytes()).await?;
    tx.write_all(bytes).await?;

    info!(hz = effective, "tickrate negotiated (inbound)");

    loop {
        let mut len_buf = [0u8; 4];
        if rx.read_exact(&mut len_buf).await.is_err() {
            break;
        }
    }

    Ok(())
}
