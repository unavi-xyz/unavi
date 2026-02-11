use avian3d::prelude::*;
use bevy::prelude::*;
use schminput::BoolActionValue;

use crate::{SqueezeDown, SqueezeUp, actions::CoreActions};

#[derive(Component)]
pub struct PrimaryRaycastInput;

pub(crate) fn read_raycast_input(
    mut commands: Commands,
    core_actions: Res<CoreActions>,
    raycaster: Query<(Entity, &RayHits), With<PrimaryRaycastInput>>,
    bool_input: Query<&BoolActionValue>,
    mut is_squeezing: Local<bool>,
    mut squeeze_target: Local<Option<Entity>>,
) {
    let Ok((pointer, ray_hits)) = raycaster.single() else {
        return;
    };

    let action = bool_input
        .get(core_actions.squeeze_right)
        .expect("squeeze action not found");

    let was_squeezing = *is_squeezing;
    *is_squeezing = action.any;

    if was_squeezing
        && !*is_squeezing
        && let Some(target) = *squeeze_target
    {
        commands
            .entity(target)
            .trigger(|entity| SqueezeUp { entity, pointer });
        *squeeze_target = None;
    }

    // Max hits should be set to 1.
    debug_assert!(ray_hits.iter().count() <= 1);

    let Some(hit) = ray_hits.iter().next() else {
        return;
    };

    // TODO render crosshair

    if !was_squeezing && *is_squeezing {
        commands
            .entity(hit.entity)
            .trigger(|entity| SqueezeDown { entity, pointer });
        *squeeze_target = Some(hit.entity);
    }
}
