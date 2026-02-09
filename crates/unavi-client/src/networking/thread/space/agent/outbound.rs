//! Agent outbound streaming - sends agent poses to a remote peer.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};

use bevy::log::{debug, info, warn};
use bytes::Bytes;
use iroh::{EndpointId, endpoint::SendStream};
use tokio::sync::watch;

use crate::networking::thread::{
    PoseState,
    space::{
        MAX_AGENT_TICKRATE,
        buffer::{CONTROL_MSG_MAX_SIZE, DATAGRAM_MAX_SIZE, SendBuffer},
        msg::{AgentIFrameMsg, AgentPFrameDatagram, ControlMsg, Datagram, read_control, write_control},
    },
};

/// Stream agent frames to a peer at the negotiated tickrate.
pub async fn stream_agent(
    tickrate: &AtomicU8,
    pose: Arc<PoseState>,
    mut iframe_stream: SendStream,
    conn: &iroh::endpoint::Connection,
    peer: EndpointId,
) -> anyhow::Result<()> {
    let mut last_iframe = 0u16;
    let mut pframe_seq = 0u16;
    let mut buf = SendBuffer::new();
    let mut datagram_buf = [0u8; DATAGRAM_MAX_SIZE];

    loop {
        let hz = tickrate.load(Ordering::Relaxed).max(1);
        let duration = Duration::from_secs_f32(1.0 / f32::from(hz));
        n0_future::time::sleep(duration).await;

        send_tick(
            &pose,
            &mut last_iframe,
            &mut pframe_seq,
            &mut buf,
            &mut iframe_stream,
            conn,
            peer,
            &mut datagram_buf,
        )
        .await?;
    }
}

#[expect(clippy::await_holding_lock, reason = "lint incorrect, we drop it")]
async fn send_tick(
    pose: &PoseState,
    last_iframe: &mut u16,
    pframe_seq: &mut u16,
    buf: &mut SendBuffer,
    iframe_stream: &mut SendStream,
    conn: &iroh::endpoint::Connection,
    peer: EndpointId,
    datagram_buf: &mut [u8],
) -> anyhow::Result<()> {
    let iframe_lock = pose.iframe.lock();

    let Some(iframe_msg) = iframe_lock.as_ref() else {
        return Ok(());
    };

    if iframe_msg.id == *last_iframe {
        drop(iframe_lock);

        let pframe_lock = pose.pframe.lock();
        if let Some(pframe_msg) = pframe_lock.as_ref() {
            *pframe_seq = pframe_seq.wrapping_add(1);
            let msg = AgentPFrameDatagram {
                iframe_id: *last_iframe,
                seq: *pframe_seq,
                pose: pframe_msg.pose.clone(),
            };
            let tagged = Datagram::AgentPFrame(msg);
            write_datagram(&tagged, conn, peer, datagram_buf)?;
        }
    } else {
        let iframe_msg = iframe_msg.clone();
        drop(iframe_lock);

        *last_iframe = iframe_msg.id;
        *pframe_seq = 0;
        *pose.pframe.lock() = None;

        write_iframe(&iframe_msg, iframe_stream, peer, &mut buf.iframe).await?;
    }

    Ok(())
}

async fn write_iframe(
    msg: &AgentIFrameMsg,
    stream: &mut SendStream,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] peer: EndpointId,
    buf: &mut [u8],
) -> anyhow::Result<()> {
    let bytes = postcard::to_slice(msg, buf)?;

    let len = u32::try_from(bytes.len())?;
    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(bytes).await?;

    #[cfg(feature = "devtools-network")]
    {
        use crate::devtools::events::{Channel, NETWORK_EVENTS, NetworkEvent};
        let _ = NETWORK_EVENTS.0.try_send(NetworkEvent::Upload {
            peer,
            bytes: bytes.len() + 4,
            channel: Channel::Stream,
        });
    }

    debug!(id = msg.id, "sent agent I-frame");

    Ok(())
}

fn write_datagram(
    msg: &Datagram,
    conn: &iroh::endpoint::Connection,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] peer: EndpointId,
    buf: &mut [u8],
) -> anyhow::Result<()> {
    let bytes = postcard::to_slice(msg, buf)?;

    if conn.datagram_send_buffer_space() < bytes.len() {
        debug!("datagram buffer full, dropping P-frame");
        return Ok(());
    }

    conn.send_datagram(Bytes::copy_from_slice(bytes))?;

    #[cfg(feature = "devtools-network")]
    {
        use crate::devtools::events::{Channel, NETWORK_EVENTS, NetworkEvent};
        let _ = NETWORK_EVENTS.0.try_send(NetworkEvent::Upload {
            peer,
            bytes: bytes.len(),
            channel: Channel::Datagram,
        });
    }

    Ok(())
}

/// Handle control stream (initiator side) - continuous loop.
/// Sends tickrate requests when `tickrate_rx` changes, handles acks.
pub async fn handle_control_stream(
    mut tx: SendStream,
    mut rx: iroh::endpoint::RecvStream,
    tickrate: &AtomicU8,
    mut tickrate_rx: watch::Receiver<u8>,
) -> anyhow::Result<()> {
    let mut buf = [0u8; CONTROL_MSG_MAX_SIZE];

    // Send initial tickrate request.
    let initial_hz = *tickrate_rx.borrow();
    write_control(&mut tx, &ControlMsg::TickrateRequest { hz: initial_hz }, &mut buf).await?;

    // Wait for ack.
    let ack: ControlMsg = read_control(&mut rx, &mut buf).await?;
    let ControlMsg::TickrateAck { hz } = ack else {
        anyhow::bail!("expected TickrateAck, got {ack:?}");
    };

    let effective = hz.min(MAX_AGENT_TICKRATE);
    tickrate.store(effective, Ordering::Relaxed);
    info!(hz = effective, "agent tickrate negotiated");

    // Continuous loop: send updates when tickrate changes.
    loop {
        tokio::select! {
            changed = tickrate_rx.changed() => {
                if changed.is_err() {
                    break;
                }
                let hz = *tickrate_rx.borrow();
                if let Err(err) = write_control(
                    &mut tx,
                    &ControlMsg::TickrateRequest { hz },
                    &mut buf,
                ).await {
                    warn!(?err, "failed to send tickrate request");
                    break;
                }
            }
            msg = read_control::<ControlMsg>(&mut rx, &mut buf) => {
                match msg {
                    Ok(ControlMsg::TickrateAck { hz }) => {
                        let effective = hz.min(MAX_AGENT_TICKRATE);
                        tickrate.store(effective, Ordering::Relaxed);
                        debug!(hz = effective, "tickrate updated");
                    }
                    Ok(ControlMsg::TickrateRequest { .. }) => {
                        // Initiator shouldn't receive requests.
                    }
                    Err(_) => break,
                }
            }
        }
    }

    Ok(())
}
