//! Distance-based tickrate adjustment for outbound agent and object data.

use bevy::prelude::*;
use unavi_locomotion::{AgentEntities, AgentRig, LocalAgent};

use crate::networking::{
    agent_receive::RemoteAgent,
    object_publish::{DynObjectId, LocallyOwned},
    thread::{
        NetworkCommand, NetworkingThread,
        space::{MAX_AGENT_TICKRATE, MAX_OBJECT_TICKRATE, MIN_TICKRATE},
    },
};

/// Distance at which tickrate reaches minimum (in meters).
const MAX_DISTANCE: f32 = 64.0;

/// Per-agent tickrate configuration.
#[derive(Component, Clone)]
pub struct AgentTickrateConfig {
    pub min: u8,
    pub max: u8,
}

impl Default for AgentTickrateConfig {
    fn default() -> Self {
        Self {
            min: MIN_TICKRATE,
            max: MAX_AGENT_TICKRATE,
        }
    }
}

/// Calculates tickrate based on distance.
/// Returns max tickrate at distance 0, min tickrate at [`MAX_DISTANCE`].
#[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn tickrate_for_distance(distance: f32, min: u8, max: u8) -> u8 {
    let t = (distance / MAX_DISTANCE).clamp(0.0, 1.0);
    let rate = f32::from(max) - t * f32::from(max - min);
    rate.round() as u8
}

/// Updates peer tickrates based on distance from local agent.
/// Runs on a timer to avoid excessive updates.
pub fn update_peer_tickrates(
    nt: Res<NetworkingThread>,
    local_agent: Query<&AgentEntities, With<LocalAgent>>,
    body_transforms: Query<&GlobalTransform, With<AgentRig>>,
    remote_agents: Query<(&RemoteAgent, &Transform, &AgentTickrateConfig)>,
) {
    let Ok(entities) = local_agent.single() else {
        return;
    };
    let Ok(local_tr) = body_transforms.get(entities.body) else {
        return;
    };
    let local_pos = local_tr.translation();

    for (remote, remote_tr, config) in &remote_agents {
        let distance = local_pos.distance(remote_tr.translation);
        let tickrate = tickrate_for_distance(distance, config.min, config.max);

        let _ = nt.command_tx.try_send(NetworkCommand::SetPeerTickrate {
            peer: remote.0,
            tickrate,
        });
    }
}

/// Updates remote object tickrates based on distance from local agent.
pub fn update_object_tickrates(
    nt: Res<NetworkingThread>,
    local_agent: Query<&AgentEntities, With<LocalAgent>>,
    body_transforms: Query<&GlobalTransform, With<AgentRig>>,
    remote_objects: Query<(&DynObjectId, &Transform), Without<LocallyOwned>>,
) {
    let Ok(entities) = local_agent.single() else {
        return;
    };
    let Ok(local_tr) = body_transforms.get(entities.body) else {
        return;
    };
    let local_pos = local_tr.translation();

    for (dyn_obj, object_tr) in &remote_objects {
        let distance = local_pos.distance(object_tr.translation);
        let tickrate = tickrate_for_distance(distance, MIN_TICKRATE, MAX_OBJECT_TICKRATE);

        let _ = nt.command_tx.try_send(NetworkCommand::SetObjectTickrate {
            object_id: dyn_obj.0,
            tickrate,
        });
    }
}
