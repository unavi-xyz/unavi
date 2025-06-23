use avian3d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::prelude::*;
use bevy_tnua::prelude::TnuaController;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

use crate::{Player, PlayerBody, PlayerCamera, PlayerHead, PlayerHeight, PlayerSpeed};

const PLAYER_RADIUS: f32 = 0.5;

/// | Group            | Height        |
/// | ---------------- | ------------- |
/// | Adult Male       | 1.70 – 1.78 m |
/// | Adult Female     | 1.60 – 1.67 m |
const DEFAULT_HEIGHT: f32 = 1.7;
const DEFAULT_SPEED: f32 = 4.0;

#[derive(Default)]
pub struct PlayerSpawner {
    pub player_height: Option<f32>,
    pub player_speed: Option<f32>,
}

impl PlayerSpawner {
    pub fn spawn(&self, commands: &mut Commands) {
        let player_height = self.player_height.unwrap_or(DEFAULT_HEIGHT);
        let player_speed = self.player_height.unwrap_or(DEFAULT_SPEED);

        let camera = commands
            .spawn((
                PlayerCamera,
                Transform::default().looking_at(Vec3::NEG_Z, Vec3::Y),
            ))
            .id();

        let mut head = commands.spawn(PlayerHead);
        head.add_child(camera);
        let head = head.id();

        let mut body = commands.spawn((
            PlayerBody,
            PlayerHeight(player_height),
            PlayerSpeed(player_speed),
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_RADIUS, player_height),
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(PLAYER_RADIUS - 0.01, 0.0)),
            LockedAxes::ROTATION_LOCKED,
        ));
        body.add_child(head);
        let body = body.id();

        let mut root = commands.spawn(Player::default());
        root.add_child(body);
    }
}
