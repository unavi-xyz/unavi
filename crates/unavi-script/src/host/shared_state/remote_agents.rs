//! Where other users' agents are, for scripts to read without waiting on the
//! world.

use std::sync::Arc;

use bevy::prelude::*;
use bevy_vrm::BoneName;
use iroh::EndpointId;
use parking_lot::RwLock;
use unavi_avatar::bones::AvatarBones;
use unavi_space::avatar_sync::RemoteAgent;

use crate::host::shared_state::near::nearest;

/// A remote agent as the engine placed it this frame, in the engine world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoteAgentSnapshot {
    pub peer: EndpointId,
    pub body: GlobalTransform,
    /// `None` until the avatar has loaded.
    pub head: Option<GlobalTransform>,
}

/// Every remote agent in a loaded space.
#[derive(Resource, Clone, Default)]
pub struct RemoteAgents(Arc<RwLock<Vec<RemoteAgentSnapshot>>>);

impl RemoteAgents {
    /// Up to [`super::near::MAX_NEAR`] agents whose body lies within `radius`
    /// of `centre` in the engine world, nearest first.
    #[must_use]
    pub fn near(&self, centre: Vec3, radius: f32) -> Vec<RemoteAgentSnapshot> {
        nearest(&self.0.read(), centre, radius, |agent| {
            agent.body.translation()
        })
    }
}

/// Copies every remote agent out of the world.
pub fn snapshot_remote_agents(
    agents: Query<(&RemoteAgent, &GlobalTransform, Option<&AvatarBones>)>,
    transforms: Query<&GlobalTransform>,
    shared: Res<RemoteAgents>,
) {
    let mut found = agents
        .iter()
        .map(|(agent, body, bones)| RemoteAgentSnapshot {
            peer: agent.0,
            body: *body,
            head: bones
                .and_then(|bones| bones.get(&BoneName::Head))
                .and_then(|head| transforms.get(*head).ok())
                .copied(),
        })
        .collect::<Vec<_>>();
    // A stable order, so an unchanged scene compares equal and writes nothing.
    found.sort_by_key(|agent| *agent.peer.as_bytes());
    if *shared.0.read() != found {
        *shared.0.write() = found;
    }
}
