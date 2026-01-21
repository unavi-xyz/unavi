use bevy::{
    pbr::{Atmosphere, AtmosphereSettings},
    prelude::*,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use loro::LoroDoc;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        .add_systems(Startup, (setup_scene, load_hsd))
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, 5.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
        Atmosphere::EARTH,
        AtmosphereSettings::default(),
    ));
}

fn load_hsd(mut commands: Commands) {
    // Create HSD.
    let doc = LoroDoc::default();

    // Load into Bevy.
}
