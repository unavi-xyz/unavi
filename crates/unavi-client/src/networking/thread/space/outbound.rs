//! Outbound connection - establishes connection and spawns streaming tasks.

use std::sync::{Arc, atomic::AtomicU8};

use bevy::log::{debug, error, info, warn};
use iroh::{EndpointAddr, EndpointId, endpoint::SendStream};
use n0_future::task::AbortOnDropHandle;
use tokio::sync::watch;

use super::{
    ALPN, MAX_AGENT_TICKRATE,
    agent::outbound::{handle_control_stream, stream_agent},
    msg::StreamInit,
    object::outbound::stream_objects,
    types::state::StateRequestMsg,
};
use crate::networking::thread::{NetworkEvent, NetworkThreadState, OutboundConn};

/// Establish an outbound connection to a peer and spawn streaming tasks.
pub async fn connect_to_peer(state: NetworkThreadState, peer: EndpointAddr) -> anyhow::Result<()> {
    info!(id = %peer.id, "connecting to peer");

    let connection = state.endpoint.connect(peer.clone(), ALPN).await?;

    // Open control bistream and send init.
    let (mut ctrl_tx, ctrl_rx) = connection.open_bi().await?;
    write_stream_init(&mut ctrl_tx, &StreamInit::AgentControl).await?;

    // Open iframe unistream and send init.
    let mut agent_iframe_stream = connection.open_uni().await?;
    write_stream_init(&mut agent_iframe_stream, &StreamInit::AgentIFrame).await?;

    let tickrate = Arc::new(AtomicU8::new(MAX_AGENT_TICKRATE));
    let (tickrate_tx, tickrate_rx) = watch::channel(MAX_AGENT_TICKRATE);

    let task = {
        let tickrate = Arc::clone(&tickrate);
        let agent_pose = Arc::clone(&state.pose);
        let object_pose = Arc::clone(&state.object_pose);
        let grabbed_rx = state.grabbed_objects_rx.clone();
        let conn = connection.clone();

        n0_future::task::spawn(async move {
            let result = tokio::select! {
                r = stream_agent(
                    &tickrate,
                    agent_pose,
                    agent_iframe_stream,
                    &conn,
                    peer.id,
                ) => r,
                r = handle_control_stream(ctrl_tx, ctrl_rx, &tickrate, tickrate_rx) => r,
                r = stream_objects(object_pose, grabbed_rx, &conn, peer.id) => r,
            };

            if let Err(err) = result {
                error!(?err, "outbound connection error");
            }

            info!(id = %peer.id, "outbound connection closed");
        })
    };

    let conn = OutboundConn {
        connection: connection.clone(),
        task: AbortOnDropHandle::new(task),
        tickrate,
        tickrate_tx,
    };

    if let Err((_, existing)) = state.outbound.insert_async(peer.id, conn).await {
        warn!(id = %peer.id, "duplicate outbound connection");
        existing.task.abort();
    }

    Ok(())
}

/// Open a `StateSync` bistream to a peer, send a request, receive their state.
pub async fn request_state_sync(
    state: &NetworkThreadState,
    connection: iroh::endpoint::Connection,
    peer_id: EndpointId,
) -> anyhow::Result<()> {
    let (mut send, mut recv) = connection.open_bi().await?;
    write_stream_init(&mut send, &StreamInit::StateSync).await?;

    // Send empty request marker.
    let req_bytes = postcard::to_stdvec(&StateRequestMsg)?;
    let req_len = u32::try_from(req_bytes.len())?;
    send.write_all(&req_len.to_le_bytes()).await?;
    send.write_all(&req_bytes).await?;
    send.finish()?;

    // Read response.
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    anyhow::ensure!(len <= 65536, "state response too large: {len}");
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await?;

    let peer_state = postcard::from_bytes(&buf)?;
    debug!(%peer_id, "received state sync response");

    let _ = state.event_tx.try_send(NetworkEvent::PeerStateReceived {
        peer: peer_id,
        state: peer_state,
    });

    Ok(())
}

/// Write a `StreamInit` message on a stream.
pub async fn write_stream_init(stream: &mut SendStream, init: &StreamInit) -> anyhow::Result<()> {
    let mut buf = [0u8; 64];
    let bytes = postcard::to_slice(init, &mut buf)?;
    let len = u32::try_from(bytes.len())?;
    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(bytes).await?;
    Ok(())
}
