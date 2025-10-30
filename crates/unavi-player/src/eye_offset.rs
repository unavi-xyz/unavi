use bevy::prelude::*;
use bevy_vrm::{BoneName, VrmScene};

use crate::{RealHeight, first_person::FirstPerson};

#[derive(Component, Deref, DerefMut)]
pub struct EyeOffset(pub Vec3);

pub(crate) fn calc_eye_offset(
    mut commands: Commands,
    mut scene_assets: ResMut<Assets<Scene>>,
    mut to_calc: Local<Vec<(Entity, f32)>>,
    mut to_remove: Local<Vec<Entity>>,
    new_scenes: Query<(Entity, &RealHeight), (With<FirstPerson>, Added<VrmScene>)>,
    vrm_scenes: Query<&VrmScene>,
) {
    for (ent, height) in new_scenes.iter() {
        to_calc.push((ent, height.0));
    }

    for (ent, height) in to_calc.iter() {
        let vrm_scene = vrm_scenes.get(*ent).expect("Scene handle not found");

        let Some(scene) = scene_assets.get_mut(vrm_scene.0.id()) else {
            // Asset might not be loaded yet.
            continue;
        };

        let mut bones = scene.world.query::<(Entity, &BoneName)>();

        let mut left_eye = None;
        let mut right_eye = None;
        let mut head = None;

        for (bone_ent, bone_name) in bones.iter(&scene.world) {
            if *bone_name == BoneName::LeftEye {
                left_eye = Some(bone_ent);
            }
            if *bone_name == BoneName::RightEye {
                right_eye = Some(bone_ent);
            }
            if *bone_name == BoneName::Head {
                head = Some(bone_ent);
            }
        }

        let mut offset = if let Some(left_eye) = left_eye
            && let Some(right_eye) = right_eye
        {
            let left_tr = scene
                .world
                .entity(left_eye)
                .get::<GlobalTransform>()
                .unwrap();
            let right_tr = scene
                .world
                .entity(right_eye)
                .get::<GlobalTransform>()
                .unwrap();

            (left_tr.translation() + right_tr.translation()) / 2.0
        } else {
            let head_tr = scene
                .world
                .entity(head.unwrap())
                .get::<GlobalTransform>()
                .unwrap();

            head_tr.translation()
        };

        offset.y += 0.08 - height / 2.0;
        offset.z -= 0.08;

        commands.entity(*ent).insert(EyeOffset(offset));

        to_remove.push(*ent);
    }

    for ent in to_remove.iter() {
        let new_calc = to_calc
            .iter()
            .copied()
            .filter(|(x, _)| x == ent)
            .collect::<Vec<_>>();
        *to_calc = new_calc;
    }

    to_remove.clear();
}
