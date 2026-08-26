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
use unavi_physics::shape;

use crate::{
    AgentRig,
    LocalAgentEntities,
    config::{
        AgentConfig,
        WorldScale,
    },
    tracking::TrackedPose,
};

#[derive(Component)]
pub struct EyeOffsetProcessed;

const DEFAULT_EYE_OFFSET_PCT: f32 = 1.05;

pub fn setup_vrm_eye_offset(
    mut commands: Commands,
    avatars: Query<(Entity, &AvatarBones, &ChildOf), (With<Avatar>, Without<EyeOffsetProcessed>)>,
    rigs: Query<&ChildOf, With<AgentRig>>,
    mut local_agents: Query<(&mut AgentConfig, &LocalAgentEntities)>,
    mut transforms: Query<&mut Transform>,
    mut tracked_poses: Query<&mut TrackedPose>,
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
            (bones.get(head).map_or(0.0, |t| t.translation().y) - avatar_y) * DEFAULT_EYE_OFFSET_PCT
        } else {
            warn!("No eye or head bones found for avatar, using fallback height");
            config.real_height / 2.0
        };

        let shoulder_width = if let Some(left_pos) = left_shoulder
            && let Some(right_pos) = right_shoulder
        {
            left_pos.distance(right_pos)
        } else {
            config.effective_vrm_radius() * 2.0
        };

        let vrm_height = eye_y;
        let vrm_radius = (shoulder_width / 2.0) * 1.5;

        config.vrm_height = Some(vrm_height);
        config.vrm_radius = Some(vrm_radius);

        let _world_scale = WorldScale::new(config.real_height, vrm_height);
        // TODO: Properly calculate and apply world scale.

        let float_height = config.float_height();
        let avatar_y_in_rig = -float_height - lowest_y;
        let head_y_in_rig = vrm_height - float_height;

        if let Ok(mut avatar_transform) = transforms.get_mut(avatar_ent) {
            avatar_transform.translation.y = avatar_y_in_rig;
        } else {
            warn!("Failed to get avatar transform for {:?}", avatar_ent);
        }

        if let Ok(mut head_pose) = tracked_poses.get_mut(entities.tracked_head) {
            head_pose.translation.y = head_y_in_rig;
        } else {
            warn!("Failed to get tracked head transform");
        }

        let rig_entity = avatar_parent.parent();
        let capsule_height = 2.0f32.mul_add(-vrm_radius, vrm_height);
        let total_height = vrm_height;

        swap_rig_shapes(
            rig_entity,
            vrm_radius,
            capsule_height,
            &mut colliders,
            &mut sensor_shapes,
        );

        if let Ok(mut rig_transform) = transforms.get_mut(rig_entity) {
            rig_transform.translation.y = total_height / 2.0;
        } else {
            warn!("Failed to update rig transform for {:?}", rig_entity);
        }

        commands
            .entity(avatar_ent)
            .trigger(|entity| SetupFirstPerson {
                entity,
                render_layers: None,
            })
            .insert(EyeOffsetProcessed);
    }
}

/// A VRM whose eye, head or shoulder bones are missing or coincident yields a
/// NaN or zero capsule, which reaches the solver as a NaN pose.
fn swap_rig_shapes(
    rig: Entity,
    radius: f32,
    height: f32,
    colliders: &mut Query<&mut Collider, With<AgentRig>>,
    sensor_shapes: &mut Query<&mut TnuaAvian3dSensorShape, With<AgentRig>>,
) {
    let Some((capsule, sensor_shape)) =
        shape::capsule(radius, height).zip(shape::cylinder(radius - 0.01, 0.0))
    else {
        warn!(
            radius,
            height, "Avatar geometry gives no usable rig collider, keeping the current one"
        );
        return;
    };

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
        tracking::{
            TrackedHead,
            TrackedPose,
        },
    };

    fn spawn_rig(app: &mut App, avatar_y: f32) -> (Entity, Entity) {
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
            .spawn((
                TrackedHead,
                TrackedPose::default(),
                Transform::default(),
                ChildOf(rig),
            ))
            .id();
        app.world_mut()
            .entity_mut(agent)
            .insert(LocalAgentEntities {
                body: rig,
                tracked_head,
            });

        let bones = [
            (BoneName::Hips, Vec3::new(0.0, 1.0, 0.0)),
            (BoneName::Head, Vec3::new(0.0, 1.6, 0.0)),
            (BoneName::LeftFoot, Vec3::new(0.0, 0.0, 0.0)),
            (BoneName::RightFoot, Vec3::new(0.0, 0.0, 0.0)),
            (BoneName::LeftShoulder, Vec3::new(-0.2, 1.4, 0.0)),
            (BoneName::RightShoulder, Vec3::new(0.2, 1.4, 0.0)),
        ];
        let mut bone_map = HashMap::new();
        for (name, local) in bones {
            let bone = app
                .world_mut()
                .spawn((
                    name,
                    Transform::from_translation(local),
                    GlobalTransform::default(),
                    ChildOf(avatar),
                ))
                .id();
            bone_map.insert(name, bone);
        }
        app.world_mut()
            .entity_mut(avatar)
            .insert(AvatarBones(bone_map));

        (agent, avatar)
    }

    fn run_offset(app: &mut App, agent: Entity, avatar: Entity) -> (f32, f32) {
        app.update();
        app.world_mut()
            .run_system_once(setup_vrm_eye_offset)
            .expect("run eye offset once");
        let transform = app
            .world()
            .get::<Transform>(avatar)
            .expect("avatar transform");
        let config = app.world().get::<AgentConfig>(agent).expect("config");
        (
            config.vrm_height.expect("vrm height"),
            transform.translation.y,
        )
    }

    #[test]
    fn eye_offset_is_independent_of_world_position() {
        let limbo = {
            let mut app = App::new();
            app.add_plugins((TransformPlugin, StatesPlugin));
            let (agent, avatar) = spawn_rig(&mut app, -0.85);
            run_offset(&mut app, agent, avatar)
        };
        let in_space = {
            let mut app = App::new();
            app.add_plugins((TransformPlugin, StatesPlugin));
            let (agent, avatar) = spawn_rig(&mut app, 5.0);
            run_offset(&mut app, agent, avatar)
        };
        assert!(
            (limbo.0 - in_space.0).abs() < 1.0e-4 && (limbo.1 - in_space.1).abs() < 1.0e-4,
            "offsets depend on world position: limbo {limbo:?} vs in-space {in_space:?}"
        );
    }

    #[test]
    fn eye_offset_grounds_the_avatar() {
        let mut app = App::new();
        app.add_plugins((TransformPlugin, StatesPlugin));
        let (agent, avatar) = spawn_rig(&mut app, 0.0);
        let (vrm_height, avatar_y_in_rig) = run_offset(&mut app, agent, avatar);

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
}
