//! Where the local user's camera, body and bones are, for scripts to read
//! and attach to.

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
    LocalAgentEntities,
};
use unavi_avatar::bones::AvatarBones;

use crate::host::shared_state::transforms::{
    AbsoluteNodeId,
    RegisterTransforms,
};

/// A node tracking the camera, body or a bone: its id in the transform
/// snapshot, and the entity a document attaches to.
#[derive(Clone, Copy, Debug)]
pub struct Tracked {
    pub node:   AbsoluteNodeId,
    pub entity: Entity,
}

/// A top-level node that follows the position of the agent's rig body and
/// stays world-aligned. It is not a descendant of the body: a document
/// attached to it would otherwise contribute its colliders to the dynamic
/// body, and a dynamic prim in it would nest under one.
#[derive(Component)]
pub struct BodyNode(Entity);

/// The local user's tracked nodes.
///
/// The camera and the body exist from the moment the agent spawns. The bones
/// are tracked apart, only once the avatar has loaded, which may be much later
/// or never.
#[derive(Resource, Clone, Default)]
pub struct LocalAgentNodes(Arc<RwLock<Nodes>>);

#[derive(Default)]
struct Nodes {
    camera: Option<Tracked>,
    /// Follows the agent's rig body in position only; see [`BodyNode`].
    body:   Option<Tracked>,
    bones:  Option<HashMap<BoneName, Tracked>>,
}

impl LocalAgentNodes {
    /// Whether the local user's avatar has loaded.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.0.read().bones.is_some()
    }

    #[must_use]
    pub fn camera(&self) -> Option<Tracked> {
        self.0.read().camera
    }

    #[must_use]
    pub fn body(&self) -> Option<Tracked> {
        self.0.read().body
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
/// The camera and body do not wait for the avatar: a script reading where the
/// viewer is looking has no use for bones, and an avatar that never loads must
/// not leave every such script blind.
pub fn track_local_agent(
    agents: Query<
        (
            &AgentAvatar,
            Option<&AgentCamera>,
            Option<&LocalAgentEntities>,
        ),
        With<LocalAgent>,
    >,
    avatars: Query<&AvatarBones>,
    registry: Res<LocalAgentNodes>,
    mut commands: Commands,
) {
    let Some((avatar, camera, entities)) = agents.iter().next() else {
        return;
    };
    // Read first: this runs every frame, and once everything that can be
    // tracked is, there is nothing to write.
    let wants_camera = camera.is_some() && registry.camera().is_none();
    let wants_body = entities.is_some() && registry.body().is_none();
    let wants_bones = !registry.is_ready() && avatars.contains(avatar.0);
    if !wants_camera && !wants_body && !wants_bones {
        return;
    }
    let mut nodes = registry.0.write();

    let mut track = |parent: Option<Entity>, name: String| {
        let node = AbsoluteNodeId {
            // Tracking nodes are never named across peers, so a fresh id
            // under a blank document cannot collide with a real one.
            doc:  DocId([0; 32]),
            node: PrimId::new(),
        };
        let mut entity = commands.spawn((
            Name::new(name),
            RegisterTransforms(node),
            Visibility::default(),
        ));
        if let Some(parent) = parent {
            entity.insert(ChildOf(parent));
        }
        Tracked {
            node,
            entity: entity.id(),
        }
    };

    let mut follows = None;
    if nodes.camera.is_none()
        && let Some(camera) = camera
    {
        nodes.camera = Some(track(Some(camera.0), "camera".into()));
    }
    if nodes.body.is_none()
        && let Some(entities) = entities
    {
        nodes.body = Some(track(None, "body".into()));
        follows = Some(entities.body);
    }
    if nodes.bones.is_none()
        && let Ok(bones) = avatars.get(avatar.0)
    {
        nodes.bones = Some(
            bones
                .0
                .iter()
                .map(|(name, entity)| (*name, track(Some(*entity), name.to_string())))
                .collect(),
        );
    }
    if let (Some(body), Some(follows)) = (nodes.body, follows) {
        commands.entity(body.entity).insert(BodyNode(follows));
    }
}

/// Moves each body node to its body's world position, keeping it aligned to
/// the world. Runs where the body's `Transform` is current and before
/// propagation, so the node does not trail the body.
pub fn follow_body(
    mut nodes: Query<(&BodyNode, &mut Transform)>,
    bodies: Query<(&Transform, Option<&ChildOf>), Without<BodyNode>>,
    globals: Query<&GlobalTransform>,
) {
    for (BodyNode(body), mut node) in &mut nodes {
        let Ok((local, parent)) = bodies.get(*body) else {
            continue;
        };
        let world = parent
            .and_then(|parent| globals.get(parent.parent()).ok())
            .map_or(local.translation, |parent| {
                parent.transform_point(local.translation)
            });
        if node.translation != world {
            node.translation = world;
        }
    }
}

/// Forgets the local user's nodes when their agent leaves, so a respawned
/// body is tracked afresh. The body node is not the agent's descendant, so it
/// goes explicitly, after letting go of what was attached to it.
pub fn forget_local_agent(
    _: On<Remove, LocalAgent>,
    registry: Res<LocalAgentNodes>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    let nodes = std::mem::take(&mut *registry.0.write());
    let Some(body) = nodes.body else {
        return;
    };
    if let Ok(attached) = children.get(body.entity) {
        for child in attached {
            commands.entity(*child).remove::<ChildOf>();
        }
    }
    commands.entity(body.entity).despawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<LocalAgentNodes>()
            .add_systems(Update, (track_local_agent, follow_body).chain());
        app
    }

    #[test]
    fn the_body_is_tracked_before_the_avatar_loads() {
        let mut app = app();
        let body = app
            .world_mut()
            .spawn(Transform {
                translation: Vec3::new(1.0, 2.0, 3.0),
                rotation:    Quat::from_rotation_y(1.0),
                scale:       Vec3::ONE,
            })
            .id();
        let tracked_head = app.world_mut().spawn_empty().id();
        let avatar = app.world_mut().spawn_empty().id();
        app.world_mut().spawn((
            LocalAgent,
            AgentAvatar(avatar),
            LocalAgentEntities { body, tracked_head },
        ));
        app.update();

        let nodes = app.world().resource::<LocalAgentNodes>().clone();
        app.update();
        let node = nodes.body().expect("the body is tracked").entity;
        assert!(app.world().get::<ChildOf>(node).is_none(), "top-level");
        let transform = app.world().get::<Transform>(node).expect("transform");
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(transform.rotation, Quat::IDENTITY, "world-aligned");
        assert!(!nodes.is_ready(), "the bones have not loaded");
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
