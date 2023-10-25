use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod input_map;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(input_map::InputMap::default())
            .add_systems(Startup, spawn_player)
            .add_systems(FixedUpdate, (void_teleport, move_player).chain());
    }
}

#[derive(Component)]
pub struct Player;

const PLAYER_HEIGHT: f32 = 1.6;
const PLAYER_WIDTH: f32 = 0.6;
const SPAWN: Vec3 = Vec3::new(0.0, PLAYER_HEIGHT * 2.0, 0.0);

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Camera3dBundle {
            transform: Transform::from_translation(SPAWN),
            ..Default::default()
        },
        Collider::capsule_y(PLAYER_HEIGHT / 2.0, PLAYER_WIDTH / 2.0),
        RigidBody::KinematicVelocityBased,
        KinematicCharacterController::default(),
        KinematicCharacterControllerOutput::default(),
    ));
}

const SPEED: f32 = 4.0;
const SPRINT_SPEED: f32 = SPEED * 1.5;

fn move_player(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    rapier_config: Res<RapierConfiguration>,
    input_map: Res<input_map::InputMap>,
    mut players: Query<
        (
            &mut KinematicCharacterController,
            &KinematicCharacterControllerOutput,
        ),
        With<Player>,
    >,
) {
    for (mut controller, output) in players.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keys.pressed(input_map.key_forward) {
            direction -= Vec3::Z;
        }
        if keys.pressed(input_map.key_backward) {
            direction += Vec3::Z;
        }
        if keys.pressed(input_map.key_left) {
            direction -= Vec3::X;
        }
        if keys.pressed(input_map.key_right) {
            direction += Vec3::X;
        }

        let speed = if keys.pressed(input_map.key_sprint) {
            SPRINT_SPEED
        } else {
            SPEED
        };

        let direction = direction.normalize_or_zero() * speed;

        let mut velocity = output.effective_translation / time.delta_seconds();

        velocity.x = direction.x;
        velocity.z = direction.z;

        velocity += rapier_config.gravity * time.delta_seconds();

        // TODO: Fix player getting large velocity on initial load

        let desired_translation = velocity * time.delta_seconds();

        controller.translation = desired_translation.into();
    }
}

const VOID_LEVEL: f32 = -50.0;

fn void_teleport(
    mut players: Query<(&mut Transform, &mut KinematicCharacterControllerOutput), With<Player>>,
) {
    for (mut transform, mut output) in players.iter_mut() {
        if transform.translation.y < VOID_LEVEL {
            transform.translation = SPAWN.clone();
            output.effective_translation = Vec3::ZERO;
            output.desired_translation = Vec3::ZERO;
        }
    }
}
