use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_xpbd_3d::prelude::*;

use super::look::{LookDirection, LookEntity, PitchEvent, YawEvent};

#[derive(Default)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub sprint_speed: f32,
    pub jump_height: f32,
    pub velocity: Vec3,
    pub input: InputState,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 7.0,
            sprint_speed: 10.0,
            jump_height: 2.0,
            velocity: Vec3::ZERO,
            input: InputState::default(),
        }
    }
}

#[derive(Component)]
pub struct YawTag;

#[derive(Component)]
pub struct PitchTag;

const PLAYER_HEIGHT: f32 = 1.6;
const PLAYER_WIDTH: f32 = 0.6;
const SPAWN: Vec3 = Vec3::new(0.0, PLAYER_HEIGHT * 2.0, 0.0);

pub fn spawn_player(mut commands: Commands) {
    let yaw = commands.spawn((TransformBundle::default(), YawTag)).id();
    let pitch = commands.spawn((TransformBundle::default(), PitchTag)).id();
    let camera = commands
        .spawn((Camera3dBundle::default(), LookDirection::default()))
        .id();

    let body = commands
        .spawn((
            Collider::capsule(PLAYER_HEIGHT, PLAYER_WIDTH),
            ColliderDensity(10.0),
            LinearVelocity::default(),
            LookEntity(camera),
            Player::default(),
            RigidBody::Dynamic,
            TnuaControllerBundle::default(),
            TransformBundle {
                global: GlobalTransform::from_translation(SPAWN),
                ..default()
            },
        ))
        .id();

    commands.entity(body).push_children(&[yaw]);
    commands.entity(yaw).push_children(&[pitch]);
    commands.entity(pitch).push_children(&[camera]);
}

pub fn apply_yaw(mut yaws: EventReader<YawEvent>, mut query: Query<&mut Transform, With<YawTag>>) {
    if let Some(yaw) = yaws.read().next() {
        for mut transform in query.iter_mut() {
            transform.rotation = Quat::from_rotation_y(yaw.0);
        }
    }
}

pub fn apply_pitch(
    mut pitches: EventReader<PitchEvent>,
    mut query: Query<&mut Transform, With<PitchTag>>,
) {
    if let Some(pitch) = pitches.read().next() {
        for mut transform in query.iter_mut() {
            transform.rotation = Quat::from_rotation_x(pitch.0);
        }
    }
}

pub fn move_player(
    time: Res<Time>,
    mut last_time: Local<f32>,
    mut players: Query<(&mut Player, &LookEntity, &mut TnuaController)>,
    look_directions: Query<&LookDirection>,
) {
    let xz = Vec3::new(1.0, 0.0, 1.0);

    for (mut player, look_entity, mut controller) in players.iter_mut() {
        let look_direction = match look_directions.get(look_entity.0) {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to get look direction: {:?}", e);
                continue;
            }
        };

        let forward = (look_direction.forward * xz).normalize();
        let right = (look_direction.right * xz).normalize();

        let mut move_direction = Vec3::ZERO;

        if player.input.forward {
            move_direction += forward;
        }
        if player.input.backward {
            move_direction -= forward;
        }
        if player.input.right {
            move_direction += right;
        }
        if player.input.left {
            move_direction -= right;
        }

        let speed = if player.input.sprint {
            player.sprint_speed
        } else {
            player.speed
        };

        let desired_velocity = move_direction.normalize_or_zero() * speed;

        if player.input.jump {
            controller.action(TnuaBuiltinJump {
                height: player.jump_height,
                ..default()
            });
        }

        controller.basis(TnuaBuiltinWalk {
            coyote_time: 0.2,
            desired_velocity,
            float_height: PLAYER_HEIGHT,
            ..default()
        });

        player.input = InputState::default();
    }

    *last_time = time.elapsed_seconds();
}

const VOID_LEVEL: f32 = -50.0;

pub fn void_teleport(
    mut players: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity), With<Player>>,
) {
    for (mut transform, mut linvel, mut angvel) in players.iter_mut() {
        if transform.translation.y < VOID_LEVEL {
            info!("Player fell into void! Teleporting player to spawn...");
            transform.translation = SPAWN;
            angvel.x = 0.0;
            angvel.y = 0.0;
            angvel.z = 0.0;
            linvel.x = 0.0;
            linvel.y = 0.0;
            linvel.z = 0.0;
        }
    }
}
