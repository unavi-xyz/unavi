use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_vrm::{BoneName, VrmInstanceId, first_person::SetupFirstPerson};

use crate::{
    PlayerAvatar, PlayerEntities, PlayerRig, TrackedPose,
    config::{PlayerConfig, WorldScale},
};

/// Marker to track which avatars have been processed for eye offset.
#[derive(Component)]
pub struct EyeOffsetProcessed;

/// Sets up eye offset and tracked head position based on VRM model bones.
pub fn setup_vrm_eye_offset(
    mut commands: Commands,
    scene_spawner: Res<SceneSpawner>,
    avatars: Query<
        (Entity, &VrmInstanceId, &ChildOf),
        (With<PlayerAvatar>, Without<EyeOffsetProcessed>),
    >,
    rigs: Query<&ChildOf, With<PlayerRig>>,
    mut local_players: Query<(&mut PlayerConfig, &PlayerEntities)>,
    mut transforms: Query<&mut Transform>,
    mut tracked_poses: Query<&mut TrackedPose>,
    mut colliders: Query<&mut Collider, With<PlayerRig>>,
    mut sensor_shapes: Query<&mut TnuaAvian3dSensorShape, With<PlayerRig>>,
    bones: Query<(&BoneName, &GlobalTransform)>,
) {
    for (avatar_ent, vrm_instance_id, avatar_parent) in avatars.iter() {
        if !scene_spawner.instance_is_ready(vrm_instance_id.0) {
            continue;
        }

        let Ok(rig_parent) = rigs.get(avatar_parent.parent()) else {
            continue;
        };
        let player_entity = rig_parent.parent();

        let mut left_eye = None;
        let mut right_eye = None;
        let mut head = None;
        let mut left_shoulder = None;
        let mut right_shoulder = None;
        let mut lowest_y = f32::MAX;

        for entity in scene_spawner.iter_instance_entities(vrm_instance_id.0) {
            let Ok((bone_name, bone_transform)) = bones.get(entity) else {
                continue;
            };

            let y = bone_transform.translation().y - 0.02; // Adjustment for feet mesh
            lowest_y = lowest_y.min(y);

            match bone_name {
                BoneName::LeftEye => left_eye = Some((entity, bone_transform.translation())),
                BoneName::RightEye => right_eye = Some((entity, bone_transform.translation())),
                BoneName::Head => head = Some((entity, bone_transform.translation())),
                BoneName::LeftShoulder => left_shoulder = Some(bone_transform.translation()),
                BoneName::RightShoulder => right_shoulder = Some(bone_transform.translation()),
                _ => {}
            }
        }

        let Ok((mut config, entities)) = local_players.get_mut(player_entity) else {
            continue;
        };

        // Calculate VRM eye height (avatar's visual eye level).
        let eye_y = if let Some((_, left_pos)) = left_eye
            && let Some((_, right_pos)) = right_eye
        {
            f32::midpoint(left_pos.y, right_pos.y)
        } else if let Some((_, head_pos)) = head {
            head_pos.y + 0.08
        } else {
            warn!("No eye or head bones found for avatar, using fallback height");
            config.real_height / 2.0
        };

        // Calculate VRM shoulder width for capsule radius.
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

        // Create WorldScale resource to scale world objects.
        // If VRM is taller than real_height, world shrinks so player feels taller.
        let world_scale = WorldScale::new(config.real_height, vrm_height);
        info!(
            "Setting world scale: ({:.4}m real) ({:.4}m vrm) = {:.4}",
            config.real_height, vrm_height, world_scale.0
        );
        commands.insert_resource(world_scale);
        // TODO: Apply world scale

        let avatar_y_in_rig = -vrm_height / 2.0 - lowest_y / 2.0;

        if let Ok(mut avatar_transform) = transforms.get_mut(avatar_ent) {
            avatar_transform.translation.y = avatar_y_in_rig;
        } else {
            warn!("Failed to get avatar transform for {:?}", avatar_ent);
        }

        let head_y_in_rig = avatar_y_in_rig + eye_y - lowest_y;

        if let Ok(mut head_pose) = tracked_poses.get_mut(entities.tracked_head) {
            head_pose.translation.y = head_y_in_rig;
        } else {
            warn!("Failed to get tracked head transform");
        }

        let rig_entity = avatar_parent.parent();
        let capsule_height = 2.0f32.mul_add(-vrm_radius, vrm_height);
        let total_height = vrm_height;

        if let Ok(mut collider) = colliders.get_mut(rig_entity) {
            *collider = Collider::capsule(vrm_radius, capsule_height);
        } else {
            warn!("Failed to update rig collider for {:?}", rig_entity);
        }

        if let Ok(mut sensor) = sensor_shapes.get_mut(rig_entity) {
            sensor.0 = Collider::cylinder(vrm_radius - 0.01, 0.0);
        } else {
            warn!("Failed to update sensor shape for {:?}", rig_entity);
        }

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
