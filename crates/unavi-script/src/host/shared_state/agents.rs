//! Where the local user's camera and bones are, for scripts to read and
//! attach to.

use std::sync::Arc;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_vrm::BoneName;
use hsd::id::{
    DocId,
    PrimId,
};
use parking_lot::RwLock;
use unavi_agent::{
    AgentAvatar,
    AgentCamera,
    LocalAgent,
};
use unavi_avatar::bones::AvatarBones;

use crate::host::shared_state::transforms::{
    AbsoluteNodeId,
    RegisterTransforms,
};

/// A node tracking a camera or bone: its id in the transform snapshot, and
/// the entity a document attaches to.
#[derive(Clone, Copy, Debug)]
pub struct Tracked {
    pub node:   AbsoluteNodeId,
    pub entity: Entity,
}

/// The local user's tracked nodes, once their body has loaded.
#[derive(Resource, Clone, Default)]
pub struct LocalAgentNodes(Arc<RwLock<Option<Nodes>>>);

#[derive(Default)]
struct Nodes {
    camera: Option<Tracked>,
    bones:  HashMap<BoneName, Tracked>,
}

impl LocalAgentNodes {
    /// Whether the local user's body has loaded.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.0.read().is_some()
    }

    #[must_use]
    pub fn camera(&self) -> Option<Tracked> {
        self.0.read().as_ref()?.camera
    }

    #[must_use]
    pub fn bone(&self, bone: BoneName) -> Option<Tracked> {
        self.0.read().as_ref()?.bones.get(&bone).copied()
    }
}

/// Spawns a tracking node under the local user's camera and each bone, once
/// the avatar has loaded. The nodes despawn with the agent.
pub fn track_local_agent(
    agents: Query<(&AgentAvatar, Option<&AgentCamera>), With<LocalAgent>>,
    avatars: Query<&AvatarBones>,
    registry: Res<LocalAgentNodes>,
    mut commands: Commands,
) {
    if registry.is_ready() {
        return;
    }
    let Some((avatar, camera)) = agents.iter().next() else {
        return;
    };
    let Ok(bones) = avatars.get(avatar.0) else {
        return;
    };

    let mut track = |parent: Entity, name: String| {
        let node = AbsoluteNodeId {
            // Tracking nodes are never named across peers, so a fresh id
            // under a blank document cannot collide with a real one.
            doc:  DocId([0; 32]),
            node: PrimId::new(),
        };
        let entity = commands
            .spawn((
                Name::new(name),
                RegisterTransforms(node),
                Visibility::default(),
                ChildOf(parent),
            ))
            .id();
        Tracked { node, entity }
    };

    let nodes = Nodes {
        camera: camera.map(|camera| track(camera.0, "camera".into())),
        bones:  bones
            .0
            .iter()
            .map(|(name, entity)| (*name, track(*entity, name.to_string())))
            .collect(),
    };
    *registry.0.write() = Some(nodes);
}

/// Forgets the local user's nodes when their agent leaves, so a respawned
/// body is tracked afresh.
pub fn forget_local_agent(_: On<Remove, LocalAgent>, registry: Res<LocalAgentNodes>) {
    *registry.0.write() = None;
}
