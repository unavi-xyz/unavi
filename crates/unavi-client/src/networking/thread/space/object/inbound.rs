//! Object inbound streaming - receives physics state from remote owners.
//!
//! Object ID is known from `StreamInit::Object` sent by `SpaceProtocol`.
//! I-frames are received on the dedicated bistream.
//! P-frames are received as datagrams (handled in `SpaceProtocol`).

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use bevy::log::debug;
use iroh::EndpointId;

use crate::networking::thread::{
    NetworkEvent,
    space::{
        MAX_OBJECT_TICKRATE,
        buffer::CONTROL_MSG_MAX_SIZE,
        datagram::SharedObjectBaselines,
        msg::{ObjectControlMsg, ObjectIFrameMsg},
        types::{object_id::ObjectId, physics_state::PhysicsBaseline},
    },
};

/// Inbound state for a single object stream.
#[derive(Default)]
pub struct ObjectInboundState {
    pub tickrate: AtomicU8,
    pub grabbed: AtomicBool,
}

/// Receive object stream data (I-frames and control messages).
/// Object ID is known from `StreamInit::Object`, passed by caller.
pub async fn recv_object_stream(
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    source: EndpointId,
    object_id: ObjectId,
    mut send: iroh::endpoint::SendStream,
    mut recv: iroh::endpoint::RecvStream,
    baselines: SharedObjectBaselines,
) -> anyhow::Result<()> {
    debug!(?object_id, "object stream initialized");

    let state = ObjectInboundState::default();

    // Send initial tickrate request.
    let request = ObjectControlMsg::TickrateRequest {
        hz: MAX_OBJECT_TICKRATE,
    };
    write_control(&mut send, &request).await?;

    // Receive messages (I-frames or control messages).
    loop {
        let mut len_buf = [0u8; 4];
        if recv.read_exact(&mut len_buf).await.is_err() {
            break;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        recv.read_exact(&mut buf).await?;

        // Try to parse as control message first (they're smaller).
        if len <= CONTROL_MSG_MAX_SIZE
            && let Ok(ctrl) = postcard::from_bytes::<ObjectControlMsg>(&buf)
        {
            handle_control(&ctrl, &state, &event_tx, object_id);
            continue;
        }

        // Parse as I-frame.
        let msg: ObjectIFrameMsg = postcard::from_bytes(&buf)?;
        debug!(id = msg.id, ?object_id, "received object I-frame");

        // Update baseline for P-frame decoding.
        baselines
            .write()
            .update(object_id, msg.id, PhysicsBaseline::from(&msg.state));

        // Send event with the received physics state.
        let _ = event_tx.try_send(NetworkEvent::ObjectPoseUpdate {
            source,
            objects: vec![(object_id, msg.state)],
        });
    }

    Ok(())
}

async fn write_control(
    send: &mut iroh::endpoint::SendStream,
    msg: &ObjectControlMsg,
) -> anyhow::Result<()> {
    let mut buf = [0u8; CONTROL_MSG_MAX_SIZE];
    let bytes = postcard::to_slice(msg, &mut buf)?;
    let len = u32::try_from(bytes.len())?;
    send.write_all(&len.to_le_bytes()).await?;
    send.write_all(bytes).await?;
    Ok(())
}

fn handle_control(
    msg: &ObjectControlMsg,
    state: &ObjectInboundState,
    event_tx: &tokio::sync::mpsc::Sender<NetworkEvent>,
    object_id: ObjectId,
) {
    match msg {
        ObjectControlMsg::TickrateRequest { hz } => {
            let effective = (*hz).min(MAX_OBJECT_TICKRATE);
            state.tickrate.store(effective, Ordering::Relaxed);
            debug!(hz = effective, ?object_id, "object tickrate negotiated");
        }
        ObjectControlMsg::TickrateAck { hz } => {
            state.tickrate.store(*hz, Ordering::Relaxed);
        }
        ObjectControlMsg::GrabStateChanged { grabbed } => {
            let prev = state.grabbed.swap(*grabbed, Ordering::Relaxed);
            if prev != *grabbed {
                let _ = event_tx.try_send(NetworkEvent::ObjectGrabChanged {
                    object_id,
                    grabbed: *grabbed,
                });
            }
        }
    }
}
