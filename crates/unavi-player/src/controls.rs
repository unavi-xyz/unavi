use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;

use crate::{Player, PLAYER_HEIGHT, SPAWN};

use super::look::{CameraLookEvent, LookDirection};

#[derive(Default)]
pub struct InputState {
    pub menu: bool,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
}

const CAM_LERP_FACTOR: f32 = 30.0;

pub fn apply_camera_look(
    mut cameras: Query<&mut Transform, With<Camera>>,
    mut events: EventReader<CameraLookEvent>,
    mut players: Query<(&mut Transform, &Children), (With<Player>, Without<Camera>)>,
    mut target_pitch: Local<Quat>,
    mut target_yaw: Local<Quat>,
    time: Res<Time>,
) {
    if let Some(look) = events.read().next() {
        *target_yaw = Quat::from_rotation_y(look.x);
        *target_pitch = Quat::from_rotation_x(look.y);
    }

    let s = time.delta_seconds() * CAM_LERP_FACTOR;

    for (mut player_tr, children) in players.iter_mut() {
        player_tr.rotation = player_tr.rotation.lerp(*target_yaw, s);

        for child in children.iter() {
            if let Ok(mut camera_tr) = cameras.get_mut(*child) {
                camera_tr.rotation = camera_tr.rotation.lerp(*target_pitch, s);
            }
        }
    }
}

const FLOAT_HEIGHT: f32 = (PLAYER_HEIGHT / 2.0) + 0.1;

pub fn move_player(
    look_direction: Res<LookDirection>,
    mut last_time: Local<f32>,
    mut players: Query<(&mut Player, &mut TnuaController)>,
    time: Res<Time>,
) {
    let xz = Vec3::new(1.0, 0.0, 1.0);

    for (mut player, mut controller) in players.iter_mut() {
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

        let desired_velocity = move_direction.normalize_or_zero() * player.speed;

        if player.input.jump {
            controller.action(TnuaBuiltinJump {
                height: player.jump_height,
                ..default()
            });
        }

        controller.basis(TnuaBuiltinWalk {
            coyote_time: 0.2,
            desired_velocity,
            float_height: FLOAT_HEIGHT,
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
