//! Sizes the rig to the avatar it is wearing, from the VRM's own bone
//! positions. Untrusted (the VRM is user-supplied): see
//! [`crate::config::RigDimensions`] for the validation gate every measured
//! size passes through before it can touch [`AgentConfig`] or a transform.

use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_vrm::{
    BoneName,
    first_person::SetupFirstPerson,
};
use unavi_avatar::{
    Avatar,
    bones::AvatarBones,
};

use crate::{
    AgentRig,
    LocalAgentEntities,
    config::{
        AgentConfig,
        RigDimensions,
    },
};

/// Marks an avatar whose rig has already been sized. Set once validation
/// runs, whether or not the measured size passed, so a persistently bad VRM
/// is not re-measured (and re-warned about) every frame.
#[derive(Component)]
pub struct RigFitted;

/// A head or eye bone's measured height is scaled by this to estimate eye
/// height when no eye bones are present (VRM 0 avatars often lack them).
const HEAD_TO_EYE_HEIGHT_PCT: f32 = 1.05;

pub fn fit_rig_to_avatar(
    mut commands: Commands,
    avatars: Query<(Entity, &AvatarBones, &ChildOf), (With<Avatar>, Without<RigFitted>)>,
    rigs: Query<&ChildOf, With<AgentRig>>,
    mut local_agents: Query<(&mut AgentConfig, &LocalAgentEntities)>,
    mut transforms: Query<&mut Transform>,
    mut colliders: Query<&mut Collider, With<AgentRig>>,
    mut sensor_shapes: Query<&mut TnuaAvian3dSensorShape, With<AgentRig>>,
    globals: Query<&GlobalTransform>,
    bones: Query<&GlobalTransform, With<BoneName>>,
) {
    for (avatar_ent, avatar_bones, avatar_parent) in avatars.iter() {
        let Ok(rig_parent) = rigs.get(avatar_parent.parent()) else {
            continue;
        };
        let agent_entity = rig_parent.parent();

        // Proportions are measured against the avatar root, so the result does
        // not depend on where the agent is standing when the rig loads.
        let Ok(avatar_y) = globals.get(avatar_ent).map(|t| t.translation().y) else {
            continue;
        };

        let mut left_eye = None;
        let mut right_eye = None;
        let mut head = None;
        let mut left_shoulder = None;
        let mut right_shoulder = None;
        let mut lowest_y = f32::MAX;

        for (bone_name, &entity) in avatar_bones.iter() {
            let Ok(bone_transform) = bones.get(entity) else {
                continue;
            };

            let y = bone_transform.translation().y - avatar_y - 0.02; // Adjustment for feet mesh.
            lowest_y = lowest_y.min(y);

            match bone_name {
                BoneName::LeftEye => left_eye = Some(entity),
                BoneName::RightEye => right_eye = Some(entity),
                BoneName::Head => head = Some(entity),
                BoneName::LeftShoulder => left_shoulder = Some(bone_transform.translation()),
                BoneName::RightShoulder => right_shoulder = Some(bone_transform.translation()),
                _ => {}
            }
        }

        let Ok((mut config, entities)) = local_agents.get_mut(agent_entity) else {
            continue;
        };

        let eye_y = if let (Some(left), Some(right)) = (left_eye, right_eye) {
            f32::midpoint(
                bones.get(left).map_or(0.0, |t| t.translation().y) - avatar_y,
                bones.get(right).map_or(0.0, |t| t.translation().y) - avatar_y,
            )
        } else if let Some(head) = head {
            (bones.get(head).map_or(0.0, |t| t.translation().y) - avatar_y) * HEAD_TO_EYE_HEIGHT_PCT
        } else {
            warn!("No eye or head bones found for avatar, using fallback height");
            config.real_height / 2.0
        };

        let shoulder_width = if let Some(left_pos) = left_shoulder
            && let Some(right_pos) = right_shoulder
        {
            left_pos.distance(right_pos)
        } else {
            config.rig_radius() * 2.0
        };

        let height = eye_y;
        let radius = (shoulder_width / 2.0) * 1.5;

        // `RigDimensions::new` warns and returns `None` for a degenerate
        // measurement; bail out of sizing (but still mark the avatar fitted
        // below) before touching the config, a collider, or a transform.
        if let Some(dims) = RigDimensions::new(height, radius) {
            config.set_rig(dims);

            let float_height = config.float_height();
            let avatar_y_in_rig = -float_height - lowest_y;
            let head_y_in_rig = dims.height - float_height;

            if let Ok(mut avatar_transform) = transforms.get_mut(avatar_ent) {
                avatar_transform.translation.y = avatar_y_in_rig;
            } else {
                warn!("Failed to get avatar transform for {avatar_ent:?}");
            }

            if let Ok(mut head_transform) = transforms.get_mut(entities.tracked_head) {
                head_transform.translation.y = head_y_in_rig;
            } else {
                warn!("Failed to get tracked head transform");
            }

            let rig_entity = avatar_parent.parent();
            let (capsule, sensor_shape) = config.rig_shapes();
            swap_rig_shapes(
                rig_entity,
                capsule,
                sensor_shape,
                &mut colliders,
                &mut sensor_shapes,
            );

            if let Ok(mut rig_transform) = transforms.get_mut(rig_entity) {
                rig_transform.translation.y = dims.height / 2.0;
            } else {
                warn!("Failed to update rig transform for {rig_entity:?}");
            }
        }

        commands
            .entity(avatar_ent)
            .trigger(|entity| SetupFirstPerson {
                entity,
                render_layers: None,
            })
            .insert(RigFitted);
    }
}

fn swap_rig_shapes(
    rig: Entity,
    capsule: Collider,
    sensor_shape: Collider,
    colliders: &mut Query<&mut Collider, With<AgentRig>>,
    sensor_shapes: &mut Query<&mut TnuaAvian3dSensorShape, With<AgentRig>>,
) {
    if let Ok(mut collider) = colliders.get_mut(rig) {
        *collider = capsule;
    } else {
        warn!("Failed to update rig collider for {rig:?}");
    }

    if let Ok(mut sensor) = sensor_shapes.get_mut(rig) {
        sensor.0 = sensor_shape;
    } else {
        warn!("Failed to update sensor shape for {rig:?}");
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use avian3d::prelude::Collider;
    use bevy::{
        ecs::system::RunSystemOnce,
        state::app::StatesPlugin,
        transform::TransformPlugin,
    };
    use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
    use bevy_vrm::BoneName;
    use unavi_avatar::{
        Avatar,
        bones::AvatarBones,
    };

    use super::*;
    use crate::{
        AgentRig,
        LocalAgentEntities,
        TrackedHead,
    };

    /// Spawns an agent/rig/avatar hierarchy with no bones yet; returns
    /// `(agent, rig, avatar)`.
    fn spawn_agent_rig_avatar(app: &mut App, avatar_y: f32) -> (Entity, Entity, Entity) {
        let agent = app
            .world_mut()
            .spawn((
                AgentConfig::default(),
                Transform::from_xyz(0.0, avatar_y, 0.0),
            ))
            .id();
        let rig = app
            .world_mut()
            .spawn((
                AgentRig,
                Transform::default(),
                Collider::capsule(0.4, 1.7),
                TnuaAvian3dSensorShape(Collider::cylinder(0.39, 0.0)),
                ChildOf(agent),
            ))
            .id();
        let avatar = app
            .world_mut()
            .spawn((Avatar, Transform::default(), ChildOf(rig)))
            .id();
        let tracked_head = app
            .world_mut()
            .spawn((TrackedHead, Transform::default(), ChildOf(rig)))
            .id();
        app.world_mut()
            .entity_mut(agent)
            .insert(LocalAgentEntities {
                body: rig,
                tracked_head,
            });
        (agent, rig, avatar)
    }

    fn spawn_bones(app: &mut App, avatar: Entity, bones: &[(BoneName, Vec3)]) {
        let mut bone_map = HashMap::new();
        for (name, local) in bones {
            let bone = app
                .world_mut()
                .spawn((
                    *name,
                    Transform::from_translation(*local),
                    GlobalTransform::default(),
                    ChildOf(avatar),
                ))
                .id();
            bone_map.insert(*name, bone);
        }
        app.world_mut()
            .entity_mut(avatar)
            .insert(AvatarBones(bone_map));
    }

    const SANE_BONES: &[(BoneName, Vec3)] = &[
        (BoneName::Hips, Vec3::new(0.0, 1.0, 0.0)),
        (BoneName::Head, Vec3::new(0.0, 1.6, 0.0)),
        (BoneName::LeftFoot, Vec3::new(0.0, 0.0, 0.0)),
        (BoneName::RightFoot, Vec3::new(0.0, 0.0, 0.0)),
        (BoneName::LeftShoulder, Vec3::new(-0.2, 1.4, 0.0)),
        (BoneName::RightShoulder, Vec3::new(0.2, 1.4, 0.0)),
    ];

    fn spawn_rig(app: &mut App, avatar_y: f32) -> (Entity, Entity) {
        let (agent, _rig, avatar) = spawn_agent_rig_avatar(app, avatar_y);
        spawn_bones(app, avatar, SANE_BONES);
        (agent, avatar)
    }

    /// Runs `fit_rig_to_avatar` once and returns the measured rig height and
    /// the avatar's resulting translation. Only used with [`SANE_BONES`],
    /// which always validate.
    fn run_fit(app: &mut App, agent: Entity, avatar: Entity) -> (f32, f32) {
        app.update();
        app.world_mut()
            .run_system_once(fit_rig_to_avatar)
            .expect("run fit once");
        let transform = app
            .world()
            .get::<Transform>(avatar)
            .expect("avatar transform");
        let config = app.world().get::<AgentConfig>(agent).expect("config");
        (config.rig_height(), transform.translation.y)
    }

    #[test]
    fn rig_fit_is_independent_of_world_position() {
        let limbo = {
            let mut app = App::new();
            app.add_plugins((TransformPlugin, StatesPlugin));
            let (agent, avatar) = spawn_rig(&mut app, -0.85);
            run_fit(&mut app, agent, avatar)
        };
        let in_space = {
            let mut app = App::new();
            app.add_plugins((TransformPlugin, StatesPlugin));
            let (agent, avatar) = spawn_rig(&mut app, 5.0);
            run_fit(&mut app, agent, avatar)
        };
        assert!(
            (limbo.0 - in_space.0).abs() < 1.0e-4 && (limbo.1 - in_space.1).abs() < 1.0e-4,
            "offsets depend on world position: limbo {limbo:?} vs in-space {in_space:?}"
        );
    }

    #[test]
    fn rig_fit_grounds_the_avatar() {
        let mut app = App::new();
        app.add_plugins((TransformPlugin, StatesPlugin));
        let (agent, avatar) = spawn_rig(&mut app, 0.0);
        let (vrm_height, avatar_y_in_rig) = run_fit(&mut app, agent, avatar);

        let config = app.world().get::<AgentConfig>(agent).expect("config");
        let float_height = config.float_height();
        let feet_y = float_height + avatar_y_in_rig;
        assert!(
            feet_y.abs() < 0.05,
            "feet at {feet_y} should touch the floor"
        );

        let head_y_in_rig = vrm_height - float_height;
        let camera_y = float_height + head_y_in_rig;
        assert!((camera_y - vrm_height).abs() < 1.0e-4);
    }

    /// Runs `fit_rig_to_avatar` once over `bones` and asserts the rig stays
    /// at its default size and every transform stays finite: the shared
    /// shape for every degenerate-geometry case below.
    fn assert_bones_are_rejected(bones: &[(BoneName, Vec3)]) {
        let mut app = App::new();
        app.add_plugins((TransformPlugin, StatesPlugin));

        let (agent, rig, avatar) = spawn_agent_rig_avatar(&mut app, 0.0);
        spawn_bones(&mut app, avatar, bones);

        app.update();
        app.world_mut()
            .run_system_once(fit_rig_to_avatar)
            .expect("run fit once");

        let config = app.world().get::<AgentConfig>(agent).expect("config");
        assert_eq!(
            config.rig_height(),
            1.7,
            "height should stay at the default"
        );
        assert_eq!(
            config.rig_radius(),
            0.4,
            "radius should stay at the default"
        );

        let avatar_transform = app.world().get::<Transform>(avatar).expect("avatar");
        assert!(avatar_transform.translation.is_finite());
        let rig_transform = app.world().get::<Transform>(rig).expect("rig");
        assert!(rig_transform.translation.is_finite());

        // The collider is still the one spawn_agent_rig_avatar set; fetching
        // it at all would panic if `fit_rig_to_avatar` had parked or removed
        // it on a validation failure.
        let _ = app.world().get::<Collider>(rig).expect("collider");
    }

    /// Coincident eye/shoulder bones (every bone at the origin) measure a
    /// zero height and radius, which must be rejected before it reaches the
    /// config, the collider, or any transform.
    #[test]
    fn zero_bones_leave_the_rig_at_its_defaults() {
        assert_bones_are_rejected(&[
            (BoneName::LeftEye, Vec3::ZERO),
            (BoneName::RightEye, Vec3::ZERO),
            (BoneName::LeftShoulder, Vec3::ZERO),
            (BoneName::RightShoulder, Vec3::ZERO),
        ]);
    }

    #[test]
    fn nan_eye_bones_leave_the_rig_at_its_defaults() {
        assert_bones_are_rejected(&[
            (BoneName::LeftEye, Vec3::new(f32::NAN, f32::NAN, f32::NAN)),
            (BoneName::RightEye, Vec3::new(f32::NAN, f32::NAN, f32::NAN)),
            (BoneName::LeftShoulder, Vec3::new(-0.2, 1.4, 0.0)),
            (BoneName::RightShoulder, Vec3::new(0.2, 1.4, 0.0)),
        ]);
    }

    #[test]
    fn zero_shoulder_width_leaves_the_rig_at_its_defaults() {
        // Both shoulders at the same point: a zero radius, which
        // `positive_length` rejects the same way it would a negative
        // one.
        assert_bones_are_rejected(&[
            (BoneName::LeftEye, Vec3::new(0.0, 1.6, 0.0)),
            (BoneName::RightEye, Vec3::new(0.0, 1.6, 0.0)),
            (BoneName::LeftShoulder, Vec3::ZERO),
            (BoneName::RightShoulder, Vec3::ZERO),
        ]);
    }

    #[test]
    fn a_huge_eye_height_leaves_the_rig_at_its_defaults() {
        assert_bones_are_rejected(&[
            (BoneName::LeftEye, Vec3::new(0.0, 1.0e8, 0.0)),
            (BoneName::RightEye, Vec3::new(0.0, 1.0e8, 0.0)),
            (BoneName::LeftShoulder, Vec3::new(-0.2, 1.4, 0.0)),
            (BoneName::RightShoulder, Vec3::new(0.2, 1.4, 0.0)),
        ]);
    }
}
