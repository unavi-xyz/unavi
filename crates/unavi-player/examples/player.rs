use avian3d::prelude::*;
use bevy::prelude::*;
use unavi_player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "../unavi-app/assets".to_string(),
                ..default()
            }),
            PhysicsDebugPlugin::default(),
            PhysicsPlugins::default(),
            PlayerPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, draw_gizmo)
        .run();
}

fn draw_gizmo(mut gizmos: Gizmos) {
    gizmos.axes(Transform::default(), 1.0);
}

const GROUND_SIZE: f32 = 15.0;
const GROUND_THICK: f32 = 0.2;

fn setup_scene(
    mut ambient: ResMut<AmbientLight>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    ambient.brightness = 100.0;
    ambient.color = Color::linear_rgb(0.95, 0.95, 1.0);

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.5, 10.0, -7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(
                GROUND_SIZE,
                GROUND_THICK,
                GROUND_SIZE,
            ))),
            material: materials.add(StandardMaterial::default()),
            transform: Transform::from_xyz(0.0, -GROUND_THICK / 2.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(GROUND_SIZE, GROUND_THICK, GROUND_SIZE),
    ));
}
