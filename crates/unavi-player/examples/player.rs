use avian3d::{
    prelude::{Collider, RigidBody},
    PhysicsPlugins,
};
use bevy::{
    color::palettes::tailwind::{BLUE_400, GRAY_200},
    prelude::*,
};
use unavi_input::InputPlugin;
use unavi_player::{PlayerPlugin, PlayerSpawner};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            InputPlugin,
            PlayerPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Player
    PlayerSpawner::default().spawn(&mut commands);

    // Lighting
    commands.spawn((PointLight::default(), Transform::from_xyz(5.0, 5.0, 5.0)));
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::default().looking_at(-Vec3::Y, Vec3::Z),
    ));

    // Ground
    commands.spawn((
        Collider::half_space(Vec3::Y),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(32.0, 32.0))),
        MeshMaterial3d(materials.add(Color::from(GRAY_200))),
        RigidBody::Static,
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));

    // Platform
    commands.spawn((
        Collider::cuboid(4.0, 1.0, 4.0),
        Mesh3d(meshes.add(Cuboid::new(4.0, 1.0, 4.0))),
        MeshMaterial3d(materials.add(Color::from(BLUE_400))),
        RigidBody::Static,
        Transform::from_xyz(-3.0, 0.0, -6.0),
    ));
}
