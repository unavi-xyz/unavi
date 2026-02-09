//! Outbound connection - establishes connection and spawns streaming tasks.

use std::sync::{Arc, atomic::AtomicU8};

use bevy::log::{error, info, warn};
use iroh::{EndpointAddr, endpoint::SendStream};
use n0_future::task::AbortOnDropHandle;

use super::{
    ALPN, MAX_AGENT_TICKRATE,
    agent::outbound::{negotiate_tickrate, stream_agent},
    msg::StreamInit,
    object::outbound::stream_objects,
};
use crate::networking::thread::{NetworkThreadState, OutboundConn};

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
                r = negotiate_tickrate(ctrl_tx, ctrl_rx, &tickrate) => r,
                r = stream_objects(object_pose, grabbed_rx, &conn, peer.id) => r,
            };

            if let Err(err) = result {
                error!(?err, "outbound connection error");
            }

            info!(id = %peer.id, "outbound connection closed");
        })
    };

    let conn = OutboundConn {
        task: AbortOnDropHandle::new(task),
        tickrate,
        connection,
    };

    if let Err((_, existing)) = state.outbound.insert_async(peer.id, conn).await {
        warn!(id = %peer.id, "duplicate outbound connection");
        existing.task.abort();
    }

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
