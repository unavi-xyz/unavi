//! Outbound object stream handler - sends physics state to remote peers.
//!
//! For each claimed object, opens two streams:
//! 1. Control bistream for tickrate negotiation + grab state
//! 2. I-frame unistream for physics data
//!
//! P-frames are sent as datagrams (tagged with object ID for routing).

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
        msg::{
            Datagram, ObjectControlMsg, ObjectIFrameMsg, ObjectPFrameDatagram, StreamInit,
            read_control, write_control,
        },
        types::object_id::ObjectId,
    },
};

use super::super::outbound::write_stream_init;

/// Track grabbed objects for network sync.
#[derive(Resource, Default)]
pub struct LocalGrabbedObjects(pub HashSet<ObjectId>);

/// Per-object stream context.
struct ObjectStreamContext {
    ctrl_tx: iroh::endpoint::SendStream,
    ctrl_rx: iroh::endpoint::RecvStream,
    iframe_tx: iroh::endpoint::SendStream,
    /// The negotiated tickrate from remote peer.
    tickrate: u8,
    /// The last tickrate we requested.
    last_requested_tickrate: u8,
    last_iframe_id: u16,
    last_grabbed: bool,
    /// Sequence number for P-frames, reset on I-frame change.
    pframe_seq: u16,
}

/// Open control and iframe streams for a claimed object.
async fn open_object_streams(
    connection: &Connection,
    object_id: ObjectId,
    peer: EndpointId,
) -> anyhow::Result<ObjectStreamContext> {
    debug!(?object_id, ?peer, "opening object streams");

    // Open control bistream.
    let (mut ctrl_tx, ctrl_rx) = connection.open_bi().await?;
    write_stream_init(&mut ctrl_tx, &StreamInit::ObjectControl { object_id }).await?;

    // Open iframe unistream.
    let mut iframe_tx = connection.open_uni().await?;
    write_stream_init(&mut iframe_tx, &StreamInit::ObjectIFrame { object_id }).await?;

    debug!(?object_id, ?peer, "object streams opened");

    Ok(ObjectStreamContext {
        ctrl_tx,
        ctrl_rx,
        iframe_tx,
        tickrate: MAX_OBJECT_TICKRATE,
        last_requested_tickrate: MAX_OBJECT_TICKRATE,
        last_iframe_id: 0,
        last_grabbed: false,
        pframe_seq: 0,
    })
}

/// Send an object I-frame on its dedicated stream.
async fn send_object_iframe(
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

/// Try to read a control message from the recv stream (non-blocking).
async fn try_read_control(
    rx: &mut iroh::endpoint::RecvStream,
    buf: &mut [u8],
) -> Option<ObjectControlMsg> {
    match tokio::time::timeout(Duration::from_millis(1), read_control::<ObjectControlMsg>(rx, buf))
        .await
    {
        Ok(Ok(msg)) => Some(msg),
        _ => None,
    }
}

/// Handle incoming control messages and respond appropriately.
async fn handle_incoming_control(ctx: &mut ObjectStreamContext, object_id: ObjectId) {
    let mut buf = [0u8; CONTROL_MSG_MAX_SIZE];
    while let Some(msg) = try_read_control(&mut ctx.ctrl_rx, &mut buf).await {
        match msg {
            ObjectControlMsg::TickrateRequest { hz } => {
                let effective = hz.min(MAX_OBJECT_TICKRATE);
                ctx.tickrate = effective;
                debug!(hz = effective, ?object_id, "object tickrate negotiated");

                let ack = ObjectControlMsg::TickrateAck { hz: effective };
                if let Err(err) = write_control(&mut ctx.ctrl_tx, &ack, &mut buf).await {
                    warn!(?err, ?object_id, "failed to send tickrate ack");
                }
            }
            ObjectControlMsg::TickrateAck { hz } => {
                ctx.tickrate = hz;
            }
            ObjectControlMsg::GrabStateChanged { .. } => {
                // Outbound doesn't need to handle this.
            }
        }
    }
}

/// Send a control message on the object control stream.
async fn send_control(
    ctx: &mut ObjectStreamContext,
    msg: &ObjectControlMsg,
) -> anyhow::Result<()> {
    let mut buf = [0u8; CONTROL_MSG_MAX_SIZE];
    write_control(&mut ctx.ctrl_tx, msg, &mut buf).await
}

/// Stream objects to a peer.
///
/// Opens dedicated control and iframe streams for each claimed object.
/// P-frames are sent as datagrams when I-frame hasn't changed.
/// Sends grab state changes when objects are grabbed/released locally.
pub async fn stream_objects(
    object_pose: Arc<ObjectPoseState>,
    mut grabbed_rx: watch::Receiver<HashSet<ObjectId>>,
    conn: &Connection,
    peer: EndpointId,
) -> anyhow::Result<()> {
    let mut streams: HashMap<ObjectId, ObjectStreamContext> = HashMap::new();

    loop {
        let duration = Duration::from_secs_f32(1.0 / f32::from(MAX_OBJECT_TICKRATE));
        n0_future::time::sleep(duration).await;

        let grabbed_objects = grabbed_rx.borrow_and_update().clone();

        // Handle incoming control messages and send tickrate updates.
        for (object_id, ctx) in streams.iter_mut() {
            handle_incoming_control(ctx, *object_id).await;

            // Check if tickrate needs updating.
            if let Some(desired) = object_pose.tickrates.read_sync(object_id, |_, hz| *hz) {
                if ctx.last_requested_tickrate != desired {
                    let msg = ObjectControlMsg::TickrateRequest { hz: desired };
                    if let Err(err) = send_control(ctx, &msg).await {
                        warn!(?err, ?object_id, "failed to send tickrate request");
                    } else {
                        ctx.last_requested_tickrate = desired;
                    }
                }
            }
        }

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
                if let Err(err) = send_control(ctx, &msg).await {
                    warn!(?err, ?object_id, "failed to send grab state");
                    streams.remove(&object_id);
                    continue;
                }
                ctx.last_grabbed = is_grabbed;
            }

            // Check if I-frame changed.
            if let Some(ctx) = streams.get_mut(&object_id)
                && ctx.last_iframe_id == iframe.id
            {
                // I-frame unchanged - send P-frame as datagram if available.
                if let Some((_, pframe)) = pframes_to_send.iter().find(|(id, _)| *id == object_id) {
                    ctx.pframe_seq = ctx.pframe_seq.wrapping_add(1);
                    let msg = ObjectPFrameDatagram {
                        seq: ctx.pframe_seq,
                        ..pframe.clone()
                    };
                    if let Err(err) = send_object_pframe(conn, &msg, peer) {
                        warn!(?err, ?object_id, "failed to send object P-frame");
                    }
                }
                continue;
            }

            // Open streams if needed.
            let ctx = if let Some(ctx) = streams.get_mut(&object_id) {
                ctx
            } else {
                let new_ctx = open_object_streams(conn, object_id, peer).await?;
                streams.insert(object_id, new_ctx);
                let ctx = streams.get_mut(&object_id).expect("just inserted");

                // Send initial grab state if grabbed.
                if is_grabbed {
                    let msg = ObjectControlMsg::GrabStateChanged { grabbed: true };
                    if let Err(err) = send_control(ctx, &msg).await {
                        warn!(?err, ?object_id, "failed to send initial grab state");
                        streams.remove(&object_id);
                        continue;
                    }
                    ctx.last_grabbed = true;
                }

                streams.get_mut(&object_id).expect("just inserted")
            };

            // Send I-frame.
            if let Err(err) = send_object_iframe(&mut ctx.iframe_tx, &iframe, peer).await {
                warn!(?err, ?object_id, "failed to send object I-frame");
                streams.remove(&object_id);
                continue;
            }

            ctx.last_iframe_id = iframe.id;
            ctx.pframe_seq = 0;
        }
    }
}
