use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world);
    }
}

const GROUND_SIZE: f32 = 20.0;
const GROUND_THICKNESS: f32 = 0.1;
const BALL_RADIUS: f32 = 0.5;

fn setup_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(GROUND_SIZE / 2.0, GROUND_THICKNESS / 2.0, GROUND_SIZE / 2.0),
        PbrBundle {
            mesh: meshes.add(
                (shape::Box {
                    min_x: -GROUND_SIZE / 2.0,
                    max_x: GROUND_SIZE / 2.0,
                    min_y: -GROUND_THICKNESS / 2.0,
                    max_y: GROUND_THICKNESS / 2.0,
                    min_z: -GROUND_SIZE / 2.0,
                    max_z: GROUND_SIZE / 2.0,
                })
                .into(),
            ),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
            transform: Transform::from_xyz(0.0, -0.1, 0.0),
            ..Default::default()
        },
    ));

    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(BALL_RADIUS),
        Restitution::coefficient(0.7),
        PbrBundle {
            mesh: meshes.add(
                (shape::Icosphere {
                    radius: BALL_RADIUS,
                    ..Default::default()
                })
                .try_into()
                .unwrap(),
            ),
            material: materials.add(Color::rgb(0.8, 0.6, 1.0).into()),
            transform: Transform::from_xyz(0.0, 8.0, -5.0),
            ..Default::default()
        },
    ));

    commands.spawn(DirectionalLightBundle::default());
}
