use avian3d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::prelude::*;
use bevy_tnua::prelude::TnuaController;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

use crate::{Player, PlayerBody, PlayerCamera, PlayerHead, DEFAULT_HEIGHT};

const PLAYER_RADIUS: f32 = 0.5;

#[derive(Default)]
pub struct PlayerSpawner {
    pub player_height: Option<f32>,
}

impl PlayerSpawner {
    pub fn spawn(&self, commands: &mut Commands) {
        let player_height = self.player_height.unwrap_or(DEFAULT_HEIGHT);

        let camera = commands.spawn(PlayerCamera).id();

        let mut head = commands.spawn(PlayerHead);
        head.add_child(camera);
        let head = head.id();

        let mut body = commands.spawn((
            PlayerBody,
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
