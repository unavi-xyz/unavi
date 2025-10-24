use avian3d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::{
    core_pipeline::{auto_exposure::AutoExposure, bloom::Bloom},
    pbr::{Atmosphere, AtmosphereSettings},
    prelude::*,
    render::camera::Exposure,
};
use bevy_tnua::prelude::TnuaController;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_vrm::{VrmBundle, VrmInstance};

use crate::{JumpStrength, Player, PlayerBody, PlayerCamera, PlayerHead, RealHeight, WalkSpeed};

const PLAYER_RADIUS: f32 = 0.5;

/// | Group            | Height        |
/// | ---------------- | ------------- |
/// | Adult Male       | 1.70 – 1.78 m |
/// | Adult Female     | 1.60 – 1.67 m |
const DEFAULT_HEIGHT: f32 = 1.7;

const DEFAULT_JUMP: f32 = 1.5;
const DEFAULT_SPEED: f32 = 4.0;

const DEFAULT_VRM: &str = "models/default.vrm";

#[derive(Default)]
pub struct PlayerSpawner {
    pub jump_strength: Option<f32>,
    pub player_height: Option<f32>,
    pub player_speed: Option<f32>,
    pub vrm_asset: Option<String>,
}

impl PlayerSpawner {
    pub fn spawn(&self, commands: &mut Commands, asset_server: &AssetServer) {
        let jump_height = self.player_height.unwrap_or(DEFAULT_JUMP);
        let real_height = self.player_height.unwrap_or(DEFAULT_HEIGHT);
        let walk_speed = self.player_height.unwrap_or(DEFAULT_SPEED);

        let camera = commands
            .spawn((
                Camera {
                    hdr: true,
                    ..Default::default()
                },
                PlayerCamera,
                Atmosphere::EARTH,
                AtmosphereSettings::default(),
                AutoExposure {
                    range: -4.0..=8.0,
                    ..Default::default()
                },
                Exposure::SUNLIGHT,
                Bloom::OLD_SCHOOL,
                Transform::default().looking_at(Vec3::NEG_Z, Vec3::Y),
            ))
            .id();

        let mut head = commands.spawn(PlayerHead);
        head.add_child(camera);
        let head = head.id();

        let vrm_path = self.vrm_asset.as_deref().unwrap_or(DEFAULT_VRM);
        let vrm_handle = asset_server.load(vrm_path);

        let mut body = commands.spawn((
            PlayerBody,
            JumpStrength(jump_height),
            RealHeight(real_height),
            WalkSpeed(walk_speed),
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_RADIUS, real_height),
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(PLAYER_RADIUS - 0.01, 0.0)),
            LockedAxes::ROTATION_LOCKED,
            Transform::from_xyz(0.0, real_height / 5.0, 0.0),
            VrmBundle {
                vrm: VrmInstance(vrm_handle),
                ..Default::default()
            },
        ));
        body.add_child(head);
        let body = body.id();

        let mut root = commands.spawn(Player::default());
        root.add_child(body);
    }
}
