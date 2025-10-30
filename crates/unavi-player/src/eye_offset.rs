use bevy::prelude::*;
use bevy_vrm::{BoneName, VrmScene, first_person::SetupFirstPerson};

use crate::{PlayerAvatar, PlayerEntities, PlayerRig, config::PlayerConfig};

/// Marker to track which avatars have been processed for eye offset.
#[derive(Component)]
pub(crate) struct EyeOffsetProcessed;

/// Sets up eye offset and tracked head position based on VRM model bones.
pub(crate) fn setup_vrm_eye_offset(
    mut commands: Commands,
    mut scene_assets: ResMut<Assets<Scene>>,
    avatars: Query<(Entity, &VrmScene, &ChildOf), (With<PlayerAvatar>, Without<EyeOffsetProcessed>)>,
    rigs: Query<&ChildOf, With<PlayerRig>>,
    players: Query<(&PlayerConfig, &PlayerEntities)>,
    mut transforms: Query<&mut Transform>,
    mut first_person_writer: EventWriter<SetupFirstPerson>,
) {
    for (avatar_ent, vrm_scene, avatar_parent) in avatars.iter() {
        let Some(scene) = scene_assets.get_mut(vrm_scene.0.id()) else {
            continue;
        };

        let Ok(rig_parent) = rigs.get(avatar_parent.parent()) else {
            continue;
        };
        let player_entity = rig_parent.parent();

        let Ok((config, entities)) = players.get(player_entity) else {
            continue;
        };

        let mut bones = scene.world.query::<(Entity, &BoneName, &GlobalTransform)>();
        let mut left_eye = None;
        let mut right_eye = None;
        let mut head = None;
        let mut lowest_y = f32::MAX;

        for (bone_ent, bone_name, bone_transform) in bones.iter(&scene.world) {
            let y = bone_transform.translation().y;
            lowest_y = lowest_y.min(y);

            match bone_name {
                BoneName::LeftEye => left_eye = Some((bone_ent, y)),
                BoneName::RightEye => right_eye = Some((bone_ent, y)),
                BoneName::Head => head = Some((bone_ent, y)),
                _ => {}
            }
        }

        let eye_y = if let Some((_, left_y)) = left_eye
            && let Some((_, right_y)) = right_eye
        {
            (left_y + right_y) / 2.0
        } else if let Some((_, head_y)) = head {
            head_y + 0.08
        } else {
            config.real_height / 2.0
        };

        let avatar_y_in_rig = -config.real_height / 2.0 - lowest_y;

        if let Ok(mut avatar_transform) = transforms.get_mut(avatar_ent) {
            avatar_transform.translation.y = avatar_y_in_rig;
        }

        let head_y_in_rig = avatar_y_in_rig + eye_y;

        if let Ok(mut head_transform) = transforms.get_mut(entities.tracked_head) {
            head_transform.translation.y = head_y_in_rig;
        }

        first_person_writer.write(SetupFirstPerson(avatar_ent));
        commands.entity(avatar_ent).insert(EyeOffsetProcessed);
    }
}
