//! The render layer every avatar mesh draws on, so a camera can leave
//! avatars out by leaving the layer out.

use std::collections::HashMap;

use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    world_serialization::WorldInstanceReady,
};
use bevy_vrm::first_person::{
    DEFAULT_RENDER_LAYERS,
    FirstPersonFlag,
};

use crate::Avatar;

/// The layer avatar meshes draw on in place of the default layer. Cameras that
/// show avatars include it; the portal stills do not.
pub const AVATAR_RENDER_LAYER: usize = 9;

/// `bevy_vrm`'s per-flag layers with the default layer swapped for
/// [`AVATAR_RENDER_LAYER`]. Pass to `SetupFirstPerson` so a first-person
/// split avatar never lands back on the default layer.
#[must_use]
pub fn avatar_first_person_layers() -> HashMap<FirstPersonFlag, RenderLayers> {
    DEFAULT_RENDER_LAYERS
        .iter()
        .map(|(flag, layers)| (*flag, avatar_layers(layers)))
        .collect()
}

fn avatar_layers(layers: &RenderLayers) -> RenderLayers {
    if layers.intersects(&RenderLayers::layer(0)) {
        layers.clone().without(0).with(AVATAR_RENDER_LAYER)
    } else {
        layers.clone()
    }
}

/// Puts every mesh of a freshly spawned avatar on [`AVATAR_RENDER_LAYER`].
/// Meshes that already carry layers keep them, so a first-person split that
/// ran first is not undone.
pub fn assign_avatar_layers(
    ready: On<WorldInstanceReady>,
    avatars: Query<(), With<Avatar>>,
    children: Query<&Children>,
    meshes: Query<(), (With<Mesh3d>, Without<RenderLayers>)>,
    mut commands: Commands,
) {
    if !avatars.contains(ready.entity) {
        return;
    }
    for descendant in children.iter_descendants(ready.entity) {
        if meshes.contains(descendant) {
            commands
                .entity(descendant)
                .insert(RenderLayers::layer(AVATAR_RENDER_LAYER));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_flag_keeps_the_default_layer() {
        for (flag, layers) in avatar_first_person_layers() {
            assert!(
                !layers.intersects(&RenderLayers::layer(0)),
                "{flag:?} still draws on the default layer"
            );
        }
    }

    #[test]
    fn flags_that_were_visible_to_default_cameras_are_on_the_avatar_layer() {
        let layers = avatar_first_person_layers();
        for flag in [FirstPersonFlag::Auto, FirstPersonFlag::Both] {
            assert!(
                layers[&flag].intersects(&RenderLayers::layer(AVATAR_RENDER_LAYER)),
                "{flag:?} is not on the avatar layer"
            );
        }
    }
}
