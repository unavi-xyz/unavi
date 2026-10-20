//! The headset's eye cameras see what the agent's camera sees.
//!
//! `bevy_mod_openxr` spawns its eye cameras bare, on the default layer alone,
//! so without this a headset draws neither avatars nor portal planes.

use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
};
use bevy_mod_xr::camera::XrCamera;
use unavi_portal::body::PortalViewer;

/// Gives every eye camera the agent camera's layers.
pub fn sync_eye_layers(
    viewer: Query<&RenderLayers, (With<PortalViewer>, Without<XrCamera>)>,
    eyes: Query<(Entity, Option<&RenderLayers>), With<XrCamera>>,
    mut commands: Commands,
) {
    let Ok(layers) = viewer.single() else {
        return;
    };
    for (eye, current) in &eyes {
        if current != Some(layers) {
            commands.entity(eye).insert(layers.clone());
        }
    }
}
