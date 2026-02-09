//! Datagram handler for inbound P-frames (agent and object).
//!
//! Datagrams are shared across the connection and tagged by type.

use std::{collections::HashMap, sync::Arc, time::Duration};

use iroh::{EndpointId, endpoint::Connection};
use parking_lot::RwLock;
use std::sync::atomic::Ordering;
use tracing::{debug, warn};

use super::{
    msg::Datagram,
    reorder::PFrameReorderBuffer,
    types::{object_id::ObjectId, physics_state::PhysicsBaseline},
};
use crate::networking::thread::{
    InboundState, NetworkEvent,
    space::{
        msg::{AgentPFrameDatagram, ObjectPFrameDatagram},
        types::physics_state::PhysicsIFrame,
    },
};

/// Per-object baseline for P-frame delta decoding.
/// Maps object id → (iframe id, baseline).
#[derive(Default)]
pub struct ObjectBaselines(HashMap<ObjectId, (u16, PhysicsBaseline)>);

impl ObjectBaselines {
    /// Update baseline from I-frame data.
    pub fn update(&mut self, object_id: ObjectId, iframe_id: u16, baseline: PhysicsBaseline) {
        self.0.insert(object_id, (iframe_id, baseline));
    }

    /// Get baseline for an object if it matches the expected I-frame ID.
    fn get(&self, object_id: &ObjectId, iframe_id: u16) -> Option<PhysicsBaseline> {
        self.0.get(object_id).and_then(|(id, baseline)| {
            if *id == iframe_id {
                Some(*baseline)
            } else {
                None
            }
        })
    }
}

/// Shared reference to object baselines for use across tasks.
pub type SharedObjectBaselines = Arc<RwLock<ObjectBaselines>>;

/// Handle all inbound datagrams for a connection.
///
/// Routes agent P-frames to the agent state and object P-frames to the event
/// system using baseline tracking for delta decoding.
pub async fn handle_datagrams(
    conn: &Connection,
    agent_state: &InboundState,
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    object_baselines: SharedObjectBaselines,
    remote: EndpointId,
) -> anyhow::Result<()> {
    let mut agent_reorder: PFrameReorderBuffer<AgentPFrameDatagram> =
        PFrameReorderBuffer::default();
    let mut object_reorders: HashMap<ObjectId, PFrameReorderBuffer<ObjectPFrameDatagram>> =
        HashMap::new();

    loop {
        let datagram = conn.read_datagram().await?;

        #[cfg(feature = "devtools-network")]
        {
            use crate::devtools::events::{Channel, NETWORK_EVENTS, NetworkEvent};
            let _ = NETWORK_EVENTS.0.try_send(NetworkEvent::Download {
                peer: remote,
                bytes: datagram.len(),
                channel: Channel::Datagram,
            });
        }

        let tagged: Datagram = match postcard::from_bytes(&datagram) {
            Ok(d) => d,
            Err(err) => {
                warn!(?err, "invalid datagram");
                continue;
            }
        };

        match tagged {
            Datagram::AgentPFrame(msg) => {
                handle_agent_pframe(msg, agent_state, &mut agent_reorder, remote).await;
            }
            Datagram::ObjectPFrame(pframe) => {
                handle_object_pframe(pframe, &object_baselines, &event_tx, &mut object_reorders, remote);
            }
        }
    }
}

fn handle_object_pframe(
    pframe: ObjectPFrameDatagram,
    baselines: &SharedObjectBaselines,
    event_tx: &tokio::sync::mpsc::Sender<NetworkEvent>,
    reorders: &mut HashMap<ObjectId, PFrameReorderBuffer<ObjectPFrameDatagram>>,
    source: EndpointId,
) {
    debug!("incoming obj pframe");

    let Some(baseline) = baselines.read().get(&pframe.object_id, pframe.iframe_id) else {
        debug!(
            object_id = ?pframe.object_id,
            iframe_id = pframe.iframe_id,
            "dropping object P-frame: no matching baseline"
        );
        return;
    };

    let object_id = pframe.object_id;
    let reorder = reorders.entry(object_id).or_default();

    if pframe.iframe_id != reorder.iframe_id() {
        reorder.reset(pframe.iframe_id);
    }

    for ready in reorder.insert(pframe) {
        // Reconstruct full state from delta.
        let reconstructed = PhysicsIFrame {
            pos: ready.state.pos.apply_to(baseline.pos).into(),
            rot: ready.state.rot,
            vel: ready.state.vel.apply_to(baseline.vel).into(),
            ang_vel: ready.state.ang_vel.apply_to(baseline.ang_vel).into(),
        };

        let _ = event_tx.try_send(NetworkEvent::ObjectPoseUpdate {
            source,
            objects: vec![(object_id, reconstructed)],
        });
    }
}

async fn handle_agent_pframe(
    msg: AgentPFrameDatagram,
    state: &InboundState,
    reorder: &mut PFrameReorderBuffer<AgentPFrameDatagram>,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] remote: EndpointId,
) {
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
            current_iframe_id, "dropping stale agent P-frame"
        );
        return;
    }

    if msg.iframe_id != reorder.iframe_id() {
        reorder.reset(msg.iframe_id);
    }

    // Use tickrate-based timing, slightly faster to catch up to latest.
    let hz = state.tickrate.load(Ordering::Relaxed).max(1);
    let frame_duration = Duration::from_secs_f32(0.8 / f32::from(hz));

    for ready_frame in reorder.insert(msg) {
        #[cfg(feature = "devtools-network")]
        {
            use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};
            let _ = NETWORK_EVENTS
                .0
                .try_send(NetworkEvent::ValidTick { peer: remote });
        }

        *state.pose.pframe.lock() = Some(ready_frame);

        n0_future::time::sleep(frame_duration).await;
    }
}
