//! Outbound object stream handler - sends physics state to remote peers.
//!
//! For each claimed object, opens a dedicated bistream on the existing connection:
//! 1. Send `StreamInit::Object { object_id }` to identify the stream
//! 2. Send I-frames reliably on the stream (without `object_id`, it's known from init)
//! 3. Send P-frames as datagrams (tagged with object ID for routing)

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use bevy::{
    log::{debug, warn},
    prelude::Resource,
};
use bytes::Bytes;
use iroh::{EndpointId, endpoint::Connection};
use tokio::sync::watch;

use crate::networking::thread::{
    ObjectPoseState,
    space::{
        MAX_OBJECT_TICKRATE,
        buffer::{CONTROL_MSG_MAX_SIZE, DATAGRAM_MAX_SIZE, OBJECT_IFRAME_MSG_MAX_SIZE},
        msg::{Datagram, ObjectControlMsg, ObjectIFrameMsg, ObjectPFrameDatagram, StreamInit},
        types::object_id::ObjectId,
    },
};

use super::super::outbound::write_stream_init;

/// Track grabbed objects for network sync.
#[derive(Resource, Default)]
pub struct LocalGrabbedObjects(pub HashSet<ObjectId>);

/// Per-object stream context.
struct ObjectStreamContext {
    send: iroh::endpoint::SendStream,
    last_iframe_id: u16,
    last_grabbed: bool,
}

/// Open a new object stream for a claimed object on an existing connection.
pub async fn open_object_stream(
    connection: &Connection,
    object_id: ObjectId,
    peer: EndpointId,
) -> anyhow::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    debug!(?object_id, ?peer, "opening object stream");

    let (mut send, recv) = connection.open_bi().await?;

    // Send stream init with object ID.
    write_stream_init(&mut send, &StreamInit::Object { object_id }).await?;

    debug!(?object_id, ?peer, "object stream opened");

    Ok((send, recv))
}

/// Send an object I-frame on its dedicated stream.
/// Object ID is not included - it's known from the `StreamInit` sent at stream creation.
pub async fn send_object_iframe(
    stream: &mut iroh::endpoint::SendStream,
    msg: &ObjectIFrameMsg,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] peer: EndpointId,
) -> anyhow::Result<()> {
    let mut buf = [0u8; OBJECT_IFRAME_MSG_MAX_SIZE];
    let bytes = postcard::to_slice(msg, &mut buf)?;

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

    debug!(id = msg.id, "sent object I-frame");

    Ok(())
}

/// Send an object P-frame as a datagram.
/// Object ID is included in the datagram for routing (datagrams are shared).
pub fn send_object_pframe(
    connection: &Connection,
    msg: &ObjectPFrameDatagram,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] peer: EndpointId,
) -> anyhow::Result<()> {
    let mut buf = [0u8; DATAGRAM_MAX_SIZE];
    let tagged = Datagram::ObjectPFrame(msg.clone());
    let bytes = postcard::to_slice(&tagged, &mut buf)?;

    if connection.datagram_send_buffer_space() < bytes.len() {
        debug!("datagram buffer full, dropping object P-frame");
        return Ok(());
    }

    connection.send_datagram(Bytes::copy_from_slice(bytes))?;

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

/// Write a control message on an object stream.
async fn write_control(
    stream: &mut iroh::endpoint::SendStream,
    msg: &ObjectControlMsg,
) -> anyhow::Result<()> {
    let mut buf = [0u8; CONTROL_MSG_MAX_SIZE];
    let bytes = postcard::to_slice(msg, &mut buf)?;
    let len = u32::try_from(bytes.len())?;
    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(bytes).await?;
    Ok(())
}

/// Stream objects to a peer.
///
/// Opens a dedicated stream for each claimed object and sends I-frames.
/// P-frames are sent as datagrams when I-frame hasn't changed.
/// Sends grab state changes when objects are grabbed/released locally.
pub async fn stream_objects(
    object_pose: Arc<ObjectPoseState>,
    mut grabbed_rx: watch::Receiver<HashSet<ObjectId>>,
    conn: &Connection,
    peer: EndpointId,
) -> anyhow::Result<()> {
    // Track open streams and state per object.
    let mut streams: HashMap<ObjectId, ObjectStreamContext> = HashMap::new();

    loop {
        let duration = Duration::from_secs_f32(1.0 / f32::from(MAX_OBJECT_TICKRATE));
        n0_future::time::sleep(duration).await;

        // Get current grabbed objects.
        let grabbed_objects = grabbed_rx.borrow_and_update().clone();

        // Collect I-frames and P-frames to send.
        let mut iframes_to_send = Vec::new();
        let mut pframes_to_send = Vec::new();
        object_pose.iframes.iter_sync(|object_id, iframe| {
            iframes_to_send.push((*object_id, iframe.clone()));
            true
        });
        object_pose.pframes.iter_sync(|object_id, pframe| {
            pframes_to_send.push((*object_id, pframe.clone()));
            true
        });

        for (object_id, iframe) in iframes_to_send {
            let is_grabbed = grabbed_objects.contains(&object_id);

            // Check if we need to send grab state change.
            if let Some(ctx) = streams.get_mut(&object_id)
                && ctx.last_grabbed != is_grabbed
            {
                let msg = ObjectControlMsg::GrabStateChanged {
                    grabbed: is_grabbed,
                };
                if let Err(err) = write_control(&mut ctx.send, &msg).await {
                    warn!(?err, ?object_id, "failed to send grab state");
                    streams.remove(&object_id);
                    continue;
                }
                ctx.last_grabbed = is_grabbed;
            }

            // Check if I-frame changed.
            if let Some(ctx) = streams.get(&object_id)
                && ctx.last_iframe_id == iframe.id
            {
                // I-frame unchanged - send P-frame as datagram if available.
                if let Some((_, pframe)) = pframes_to_send.iter().find(|(id, _)| *id == object_id)
                    && let Err(err) = send_object_pframe(conn, pframe, peer)
                {
                    warn!(?err, ?object_id, "failed to send object P-frame");
                }
                continue;
            }

            // Open stream if needed.
            let ctx = if let Some(ctx) = streams.get_mut(&object_id) {
                ctx
            } else {
                let (send, _recv) = open_object_stream(conn, object_id, peer).await?;
                streams.insert(
                    object_id,
                    ObjectStreamContext {
                        send,
                        last_iframe_id: 0,
                        last_grabbed: false,
                    },
                );
                let ctx = streams.get_mut(&object_id).expect("just inserted");

                // Send initial grab state if grabbed.
                if is_grabbed {
                    let msg = ObjectControlMsg::GrabStateChanged { grabbed: true };
                    if let Err(err) = write_control(&mut ctx.send, &msg).await {
                        warn!(?err, ?object_id, "failed to send initial grab state");
                        streams.remove(&object_id);
                        continue;
                    }
                    ctx.last_grabbed = true;
                }

                streams.get_mut(&object_id).expect("just inserted")
            };

            // Send I-frame.
            if let Err(err) = send_object_iframe(&mut ctx.send, &iframe, peer).await {
                warn!(?err, ?object_id, "failed to send object I-frame");
                // Remove stream on error (will be reopened on next tick).
                streams.remove(&object_id);
                continue;
            }

            ctx.last_iframe_id = iframe.id;
        }
    }
}
