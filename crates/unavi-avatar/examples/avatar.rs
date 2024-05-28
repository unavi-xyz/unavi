use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use unavi_avatar::{AvatarBundle, AvatarPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin, AvatarPlugin))
        .add_systems(Startup, (setup_avatar, setup_scene))
        .run();
}

const GROUND_SIZE: f32 = 10.0;

fn setup_scene(
    mut ambient: ResMut<AmbientLight>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    ambient.brightness = 100.0;
    ambient.color = Color::rgb(0.95, 0.95, 1.0);

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(-4.5, 10.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid {
            half_size: Vec3::new(GROUND_SIZE / 2.0, 0.1, GROUND_SIZE / 2.0),
        })),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(0.0, -0.1, 0.0),
        ..default()
    });

    let mut transform = Transform::from_xyz(0.0, 3.0, 10.0);
    transform.look_at(Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 1.0, 0.0));

    commands.spawn((
        Camera3dBundle {
            transform,
            ..Default::default()
        },
        PanOrbitCamera::default(),
    ));
}

fn setup_avatar(mut commands: Commands) {
    commands.spawn((TransformBundle::default(), AvatarBundle::default()));
}
