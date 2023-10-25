use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::{
    events::{PitchEvent, YawEvent},
    look::{LookDirection, LookEntity},
};

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
            input: InputState {
                forward: false,
                backward: false,
                left: false,
                right: false,
                up: false,
                down: false,
                jump: false,
                sprint: false,
            },
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
            RigidBody::KinematicPositionBased,
            Collider::capsule_y(PLAYER_HEIGHT / 2.0, PLAYER_WIDTH / 2.0),
            TransformBundle {
                local: Transform::from_translation(SPAWN),
                ..Default::default()
            },
            Velocity::default(),
            KinematicCharacterController::default(),
            KinematicCharacterControllerOutput::default(),
        ))
        .id();

    commands.entity(body).push_children(&[yaw]);
    commands.entity(yaw).push_children(&[pitch]);
    commands.entity(pitch).push_children(&[camera]);
}

pub fn apply_yaw(mut yaws: EventReader<YawEvent>, mut query: Query<&mut Transform, With<YawTag>>) {
    if let Some(yaw) = yaws.iter().next() {
        for mut transform in query.iter_mut() {
            transform.rotation = Quat::from_rotation_y(**yaw);
        }
    }
}

pub fn apply_pitch(
    mut pitches: EventReader<PitchEvent>,
    mut query: Query<&mut Transform, With<PitchTag>>,
) {
    if let Some(pitch) = pitches.iter().next() {
        for mut transform in query.iter_mut() {
            transform.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, **pitch, 0.0);
        }
    }
}

const DAMPING_FACTOR: f32 = 0.75;

pub fn move_player(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    mut players: Query<(
        &Player,
        &LookEntity,
        &Velocity,
        &mut KinematicCharacterController,
        &KinematicCharacterControllerOutput,
    )>,
    look_directions: Query<&LookDirection>,
) {
    let xz = Vec3::new(1.0, 0.0, 1.0);

    for (player, look_entity, velocity, mut controller, output) in players.iter_mut() {
        let look_direction = look_directions
            .get_component::<LookDirection>(look_entity.0)
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
            velocity.linvel.clone() * DAMPING_FACTOR * xz
        };

        desired_velocity.y = if player.input.jump && output.grounded {
            player.jump_velocity
        } else {
            rapier_config
                .gravity
                .y
                .mul_add(time.delta_seconds(), velocity.linvel.y)
        };

        let desired_translation = desired_velocity * time.delta_seconds();
        controller.translation = desired_translation.into();
    }
}

const VOID_LEVEL: f32 = -50.0;

pub fn void_teleport(
    mut players: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut KinematicCharacterControllerOutput,
        ),
        With<Player>,
    >,
) {
    for (mut transform, mut velocity, mut output) in players.iter_mut() {
        // TODO: Fix void teleporting send you into orbit
        if transform.translation.y < VOID_LEVEL {
            transform.translation = SPAWN.clone();
            velocity.linvel = Vec3::ZERO;
            velocity.angvel = Vec3::ZERO;
            output.effective_translation = Vec3::ZERO;
            output.desired_translation = Vec3::ZERO;
        }
    }
}
