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

/// The local user's tracked nodes.
///
/// The camera and the bones are tracked apart: the camera exists from the
/// moment the agent spawns, and the bones only once the avatar has loaded,
/// which may be much later or never.
#[derive(Resource, Clone, Default)]
pub struct LocalAgentNodes(Arc<RwLock<Nodes>>);

#[derive(Default)]
struct Nodes {
    camera: Option<Tracked>,
    bones:  Option<HashMap<BoneName, Tracked>>,
}

impl LocalAgentNodes {
    /// Whether the local user's body has loaded.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.0.read().bones.is_some()
    }

    #[must_use]
    pub fn camera(&self) -> Option<Tracked> {
        self.0.read().camera
    }

    #[must_use]
    pub fn bone(&self, bone: BoneName) -> Option<Tracked> {
        self.0.read().bones.as_ref()?.get(&bone).copied()
    }
}

/// Spawns a tracking node under the local user's camera as soon as there is
/// one, and under each bone once the avatar has loaded. The nodes despawn
/// with the agent.
///
/// The camera does not wait for the body: a script reading where the viewer
/// is looking has no use for bones, and an avatar that never loads must not
/// leave every such script blind.
pub fn track_local_agent(
    agents: Query<(&AgentAvatar, Option<&AgentCamera>), With<LocalAgent>>,
    avatars: Query<&AvatarBones>,
    registry: Res<LocalAgentNodes>,
    mut commands: Commands,
) {
    let Some((avatar, camera)) = agents.iter().next() else {
        return;
    };
    // Read first: this runs every frame, and once everything that can be
    // tracked is, there is nothing to write.
    let wants_camera = camera.is_some() && registry.camera().is_none();
    let wants_bones = !registry.is_ready() && avatars.contains(avatar.0);
    if !wants_camera && !wants_bones {
        return;
    }
    let mut nodes = registry.0.write();

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

    if nodes.camera.is_none()
        && let Some(camera) = camera
    {
        nodes.camera = Some(track(camera.0, "camera".into()));
    }
    if nodes.bones.is_none()
        && let Ok(bones) = avatars.get(avatar.0)
    {
        nodes.bones = Some(
            bones
                .0
                .iter()
                .map(|(name, entity)| (*name, track(*entity, name.to_string())))
                .collect(),
        );
    }
}

/// Forgets the local user's nodes when their agent leaves, so a respawned
/// body is tracked afresh.
pub fn forget_local_agent(_: On<Remove, LocalAgent>, registry: Res<LocalAgentNodes>) {
    *registry.0.write() = Nodes::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<LocalAgentNodes>()
            .add_systems(Update, track_local_agent);
        app
    }

    #[test]
    fn the_camera_is_tracked_before_the_avatar_loads() {
        let mut app = app();
        let camera = app.world_mut().spawn_empty().id();
        let avatar = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn((LocalAgent, AgentAvatar(avatar), AgentCamera(camera)));
        app.update();

        let nodes = app.world().resource::<LocalAgentNodes>().clone();
        assert!(
            nodes.camera().is_some(),
            "the camera does not wait for bones"
        );
        assert!(!nodes.is_ready(), "the body has not loaded");

        app.world_mut()
            .entity_mut(avatar)
            .insert(AvatarBones(std::collections::HashMap::default()));
        app.update();
        assert!(nodes.is_ready(), "the bones are picked up once they arrive");
    }
}
