//! Outbound connection handler - sends our poses to a remote player.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};

use anyhow::bail;
use bevy::log::{debug, error, info, warn};
use iroh::{EndpointId, endpoint::SendStream};
use tokio::task::JoinHandle;

use super::{ControlMsg, IFrameMsg, PFrameDatagram};
use crate::networking::thread::{NetworkThreadState, OutboundConn, PoseState};

const MAX_TICKRATE: u8 = 20;

pub async fn handle_outbound(state: NetworkThreadState, remote: EndpointId) -> anyhow::Result<()> {
    info!("connecting to {remote}");

    let connection = state.endpoint.connect(remote, super::ALPN).await?;

    // Open control bistream for tickrate negotiation.
    let (ctrl_tx, ctrl_rx) = connection.open_bi().await?;

    // Open I-frame stream (unidirectional).
    let iframe_stream = connection.open_uni().await?;

    // Create channels for receiving from command dispatch.
    let tickrate = Arc::new(AtomicU8::new(MAX_TICKRATE));

    // Spawn the main task that handles all sending.
    let task: JoinHandle<()> = {
        let tickrate = Arc::clone(&tickrate);
        let pose = Arc::clone(&state.pose);
        let conn = connection.clone();

        tokio::spawn(async move {
            let result = tokio::select! {
                r = send_frames(&tickrate, pose, iframe_stream, &conn, remote) => r,
                r = handle_tickrate(ctrl_tx, ctrl_rx, &tickrate) => r,
            };

            if let Err(err) = result {
                error!(?err, "outbound connection error");
            }

            info!("outbound connection closed: {remote}");
        })
    };

    // Register in outbound map.
    let conn = OutboundConn { task };

    if let Err((_, existing)) = state.outbound.insert_async(remote, conn).await {
        warn!("duplicate outbound connection to {remote}");
        existing.task.abort();
    }

    Ok(())
}

async fn send_frames(
    tickrate: &AtomicU8,
    pose: Arc<PoseState>,
    mut iframe_stream: SendStream,
    conn: &iroh::endpoint::Connection,
    remote: EndpointId,
) -> anyhow::Result<()> {
    let mut last_iframe = 0;

    loop {
        let hz = tickrate.load(Ordering::Relaxed).max(1);
        let duration = Duration::from_secs_f32(1.0 / f32::from(hz));
        tokio::time::sleep(duration).await;

        let iframe_lock = pose.iframe.lock();
        let Some(iframe_msg) = iframe_lock.as_ref() else {
            continue;
        };

        if iframe_msg.id == last_iframe {
            drop(iframe_lock);

            let pframe_lock = pose.pframe.lock();
            if let Some(pframe_msg) = pframe_lock.as_ref() {
                send_pframe(pframe_msg, conn, remote)?;
            }
        } else {
            let iframe_msg = iframe_msg.clone();
            drop(iframe_lock);

            last_iframe = iframe_msg.id;
            send_iframe(&iframe_msg, &mut iframe_stream, remote).await?;
        }
    }
}

async fn send_iframe(
    msg: &IFrameMsg,
    stream: &mut SendStream,
    remote: EndpointId,
) -> anyhow::Result<()> {
    let bytes = postcard::to_stdvec(msg)?;

    // Write length-prefixed message.
    let len = u32::try_from(bytes.len())?;
    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(&bytes).await?;

    #[cfg(feature = "devtools-network")]
    {
        use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};
        let _ = NETWORK_EVENTS.0.send(NetworkEvent::Upload {
            peer: remote,
            bytes: bytes.len() + 4,
            is_iframe: true,
        });
    }

    debug!(id = msg.id, "sent I-frame");

    Ok(())
}

fn send_pframe(
    msg: &PFrameDatagram,
    conn: &iroh::endpoint::Connection,
    remote: EndpointId,
) -> anyhow::Result<()> {
    let bytes = postcard::to_stdvec(msg)?;

    // Check buffer space, drop if full.
    if conn.datagram_send_buffer_space() < bytes.len() {
        debug!("datagram buffer full, dropping P-frame");
        return Ok(());
    }

    conn.send_datagram(bytes.clone().into())?;

    #[cfg(feature = "devtools-network")]
    {
        use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};
        let _ = NETWORK_EVENTS.0.send(NetworkEvent::Upload {
            peer: remote,
            bytes: bytes.len(),
            is_iframe: false,
        });
    }

    Ok(())
}

/// Handles tickrate negotiation over control bistream.
async fn handle_tickrate(
    mut tx: SendStream,
    mut rx: iroh::endpoint::RecvStream,
    tickrate: &AtomicU8,
) -> anyhow::Result<()> {
    // Send our desired tickrate.
    let request = ControlMsg::TickrateRequest { hz: MAX_TICKRATE };
    let bytes = postcard::to_stdvec(&request)?;
    let len = u32::try_from(bytes.len())?;
    tx.write_all(&len.to_le_bytes()).await?;
    tx.write_all(&bytes).await?;

    // Read their acknowledgement.
    let mut len_buf = [0u8; 4];
    rx.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;

    if len > 1024 {
        bail!("control message too large: {len}");
    }

    let mut buf = vec![0u8; len];
    rx.read_exact(&mut buf).await?;

    let ack: ControlMsg = postcard::from_bytes(&buf)?;
    let ControlMsg::TickrateAck { hz } = ack else {
        bail!("expected TickrateAck, got {ack:?}");
    };

    // Use the minimum of requested and acknowledged.
    let effective = hz.min(MAX_TICKRATE);
    tickrate.store(effective, Ordering::Relaxed);
    info!(hz = effective, "tickrate negotiated");

    // Keep connection alive, wait for remote to close.
    loop {
        let mut len_buf = [0u8; 4];
        if rx.read_exact(&mut len_buf).await.is_err() {
            break;
        }
    }

    Ok(())
}
