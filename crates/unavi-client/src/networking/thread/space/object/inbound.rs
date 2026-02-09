//! Object inbound streaming - receives physics state from remote owners.
//!
//! Object ID is known from `StreamInit::ObjectControl` or `StreamInit::ObjectIFrame`.
//! Control messages are received on the control bistream.
//! I-frames are received on the dedicated unistream.
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
        msg::{ObjectControlMsg, ObjectIFrameMsg, read_control, write_control},
        types::{object_id::ObjectId, physics_state::PhysicsBaseline},
    },
};

/// Inbound state for a single object stream.
#[derive(Default)]
pub struct ObjectInboundState {
    pub tickrate: AtomicU8,
    pub grabbed: AtomicBool,
}

/// Receive object control stream (tickrate + grab state).
pub async fn recv_object_control(
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    object_id: ObjectId,
    mut tx: iroh::endpoint::SendStream,
    mut rx: iroh::endpoint::RecvStream,
) -> anyhow::Result<()> {
    debug!(?object_id, "object control stream initialized");

    let state = ObjectInboundState::default();
    let mut buf = [0u8; CONTROL_MSG_MAX_SIZE];

    // Send initial tickrate request.
    let request = ObjectControlMsg::TickrateRequest {
        hz: MAX_OBJECT_TICKRATE,
    };
    write_control(&mut tx, &request, &mut buf).await?;

    // Continuous loop: receive control messages.
    loop {
        let msg: ObjectControlMsg = read_control(&mut rx, &mut buf).await?;

        match msg {
            ObjectControlMsg::TickrateRequest { hz } => {
                let effective = hz.min(MAX_OBJECT_TICKRATE);
                state.tickrate.store(effective, Ordering::Relaxed);

                write_control(
                    &mut tx,
                    &ObjectControlMsg::TickrateAck { hz: effective },
                    &mut buf,
                )
                .await?;
                debug!(hz = effective, ?object_id, "object tickrate updated");
            }
            ObjectControlMsg::TickrateAck { hz } => {
                state.tickrate.store(hz, Ordering::Relaxed);
            }
            ObjectControlMsg::GrabStateChanged { grabbed } => {
                let prev = state.grabbed.swap(grabbed, Ordering::Relaxed);
                if prev != grabbed {
                    let _ = event_tx.try_send(NetworkEvent::ObjectGrabChanged {
                        object_id,
                        grabbed,
                    });
                }
            }
        }
    }
}

/// Receive object I-frames from the unistream.
pub async fn recv_object_iframes(
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    source: EndpointId,
    object_id: ObjectId,
    mut rx: iroh::endpoint::RecvStream,
    baselines: SharedObjectBaselines,
) -> anyhow::Result<()> {
    debug!(?object_id, "object iframe stream initialized");

    loop {
        let mut len_buf = [0u8; 4];
        if rx.read_exact(&mut len_buf).await.is_err() {
            break;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        rx.read_exact(&mut buf).await?;

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
