//! Holds the veil over the first frames, lifting it once the first view is
//! worth seeing.

use bevy::prelude::*;
use unavi_portal::view::camera::PortalViewCamera;

use crate::{
    camera::veil::Veil,
    scene::limbo::Limbo,
};

/// The shortest the first frames stay hidden, so pipelines compiling on
/// launch do not flash through.
const MIN_HOLD_SECS: f32 = 1.5;
/// The longest the first frames stay hidden, so a camera or texture that
/// never arrives cannot hold the app blind.
const MAX_HOLD_SECS: f32 = 3.0;

/// Present from launch until the startup veil lifts.
#[derive(Resource)]
pub struct StartupHold;

/// Lifts the veil once a camera exists, the limbo floor's texture is loaded
/// and [`MIN_HOLD_SECS`] have passed, or after [`MAX_HOLD_SECS`].
pub fn release_startup(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    materials: Res<Assets<StandardMaterial>>,
    cameras: Query<(), (With<Camera3d>, Without<PortalViewCamera>)>,
    floors: Query<&MeshMaterial3d<StandardMaterial>, With<Limbo>>,
    mut veil: ResMut<Veil>,
    mut commands: Commands,
) {
    let floor_ready = floors.iter().any(|floor| {
        materials
            .get(&floor.0)
            .and_then(|material| material.base_color_texture.as_ref())
            .is_some_and(|texture| asset_server.is_loaded_with_dependencies(texture.id()))
    });
    let elapsed = time.elapsed_secs();
    let ready = !cameras.is_empty() && floor_ready && elapsed >= MIN_HOLD_SECS;
    if !ready && elapsed < MAX_HOLD_SECS {
        return;
    }
    veil.reveal();
    commands.remove_resource::<StartupHold>();
}
