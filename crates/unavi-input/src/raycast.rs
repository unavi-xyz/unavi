use avian3d::prelude::*;
use bevy::prelude::*;
use schminput::BoolActionValue;

use crate::{SqueezeDown, SqueezeUp, actions::CoreActions, crosshair::Crosshair};

#[derive(Component)]
pub struct PrimaryRaycastInput;

pub(crate) fn read_raycast_input(
    mut commands: Commands,
    core_actions: Res<CoreActions>,
    raycaster: Query<(Entity, &RayCaster, &RayHits), With<PrimaryRaycastInput>>,
    bool_input: Query<&BoolActionValue>,
    mut crosshair: Query<(&mut Visibility, &mut Transform), With<Crosshair>>,
    mut is_squeezing: Local<bool>,
    mut squeeze_target: Local<Option<Entity>>,
) {
    let Ok((pointer, ray, ray_hits)) = raycaster.single() else {
        return;
    };

    let action = bool_input
        .get(core_actions.squeeze_right)
        .expect("squeeze action not found");

    let was_squeezing = *is_squeezing;
    *is_squeezing = action.any;

    if was_squeezing && !*is_squeezing {
        commands.trigger(SqueezeUp {
            entity: *squeeze_target,
            pointer,
        });
        *squeeze_target = None;
    }

    let Ok((mut crosshair_vis, mut crosshair_tr)) = crosshair.single_mut() else {
        return;
    };

    // Max hits should be set to 1.
    debug_assert!(ray_hits.iter().count() <= 1);

    let hit = ray_hits.iter().next();

    if let Some(hit) = hit {
        *crosshair_vis = Visibility::Visible;
        crosshair_tr.translation = ray.global_origin() + (ray.global_direction() * hit.distance);
        let up = arbitrary_up(hit.normal);
        *crosshair_tr = crosshair_tr.looking_to(up, hit.normal);
    } else {
        *crosshair_vis = Visibility::Hidden;
        crosshair_tr.scale = Vec3::ONE;
    }

    if !was_squeezing && *is_squeezing {
        let hit_entity = hit.map(|h| h.entity);
        commands.trigger(SqueezeDown {
            entity: hit_entity,
            pointer,
        });
        *squeeze_target = hit_entity;
    }
}

fn arbitrary_up(normal: Vec3) -> Vec3 {
    let n = normal.normalize();

    // Pick axis with smallest component magnitude.
    let reference = if n.x.abs() < n.y.abs() && n.x.abs() < n.z.abs() {
        Vec3::X
    } else if n.y.abs() < n.z.abs() {
        Vec3::Y
    } else {
        Vec3::Z
    };

    n.cross(reference).normalize()
}
