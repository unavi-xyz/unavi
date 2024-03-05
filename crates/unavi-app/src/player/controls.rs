use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_xpbd_3d::prelude::*;

use super::{
    events::{PitchEvent, YawEvent},
    look::{LookDirection, LookEntity},
};

#[derive(Default)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub jump: bool,
    pub sprint: bool,
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub sprint_speed: f32,
    pub jump_velocity: f32,
    pub velocity: Vec3,
    pub input: InputState,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 3.0,
            sprint_speed: 5.0,
            jump_velocity: 4.0,
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
            Player::default(),
            LookEntity(camera),
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_HEIGHT, PLAYER_WIDTH),
            ColliderDensity(10.0),
            TransformBundle {
                local: Transform::from_translation(SPAWN),
                ..default()
            },
            LinearVelocity::default(),
            TnuaControllerBundle::default(),
            bevy_vrm::mtoon::MtoonMainCamera,
        ))
        .id();

    commands.entity(body).push_children(&[yaw]);
    commands.entity(yaw).push_children(&[pitch]);
    commands.entity(pitch).push_children(&[camera]);
}

pub fn apply_yaw(mut yaws: EventReader<YawEvent>, mut query: Query<&mut Transform, With<YawTag>>) {
    if let Some(yaw) = yaws.read().next() {
        for mut transform in query.iter_mut() {
            transform.rotation = Quat::from_rotation_y(**yaw);
        }
    }
}

pub fn apply_pitch(
    mut pitches: EventReader<PitchEvent>,
    mut query: Query<&mut Transform, With<PitchTag>>,
) {
    if let Some(pitch) = pitches.read().next() {
        for mut transform in query.iter_mut() {
            transform.rotation = Quat::from_rotation_x(**pitch);
        }
    }
}

const DAMPING_FACTOR: f32 = 0.75;

pub fn move_player(
    time: Res<Time>,
    mut last_time: Local<f32>,
    mut players: Query<(
        &mut Player,
        &LookEntity,
        &mut TnuaController,
        &LinearVelocity,
    )>,
    look_directions: Query<&LookDirection>,
) {
    let xz = Vec3::new(1.0, 0.0, 1.0);

    for (mut player, look_entity, mut controller, velocity) in players.iter_mut() {
        let look_direction = look_directions
            .get(look_entity.0)
            .expect("Failed to get LookDirection from Entity");

        let forward = (look_direction.forward * xz).normalize();
        let right = (look_direction.right * xz).normalize();
        let up = Vec3::Y;

        let mut desired_velocity = Vec3::ZERO;

        if player.input.forward {
            desired_velocity += forward;
        }
        if player.input.backward {
            desired_velocity -= forward;
        }
        if player.input.right {
            desired_velocity += right;
        }
        if player.input.left {
            desired_velocity -= right;
        }
        if player.input.up {
            desired_velocity += up;
        }
        if player.input.down {
            desired_velocity -= up;
        }

        let speed = if player.input.sprint {
            player.sprint_speed
        } else {
            player.speed
        };

        desired_velocity = if desired_velocity.length_squared() > 1E-6 {
            desired_velocity.normalize() * speed
        } else {
            // No input, apply damping to the x/z of the current velocity
            // velocity * DAMPING_FACTOR * xz
            **velocity
        };

        if player.input.jump {
            controller.action(TnuaBuiltinJump {
                height: 4.0,
                ..default()
            });
        }

        controller.basis(TnuaBuiltinWalk {
            desired_velocity,
            float_height: 0.5,
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
            linvel.x = 0.0;
            linvel.y = 0.0;
            linvel.z = 0.0;
            angvel.x = 0.0;
            angvel.y = 0.0;
            angvel.z = 0.0;
        }
    }
}
