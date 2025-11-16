use avian3d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::{
    camera::{Exposure, visibility::RenderLayers},
    pbr::{Atmosphere, AtmosphereSettings},
    post_process::{auto_exposure::AutoExposure, bloom::Bloom},
    prelude::*,
    render::view::Hdr,
};
use bevy_tnua::prelude::TnuaController;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_vrm::{
    VrmBundle, VrmInstance,
    first_person::{FirstPersonFlag, RENDER_LAYERS},
};

use crate::{
    LocalPlayer, PlayerAvatar, PlayerCamera, PlayerEntities, PlayerRig,
    animation::{defaults::default_character_animations, velocity::AverageVelocity},
    bones::AvatarBones,
    config::{PLAYER_RADIUS, PlayerConfig},
    tracking::{TrackedHead, TrackedPose, TrackingSource},
};

const DEFAULT_AVATAR: &str =
    "https://unavi.nyc3.cdn.digitaloceanspaces.com/assets/models/default.vrm";

/// Builder for spawning a player entity.
#[derive(Default)]
pub struct PlayerSpawner {
    pub config: Option<PlayerConfig>,
    pub tracking_source: Option<TrackingSource>,
    pub vrm_asset: Option<String>,
    pub camera_active: Option<bool>,
}

impl PlayerSpawner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: PlayerConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_tracking_source(mut self, source: TrackingSource) -> Self {
        self.tracking_source = Some(source);
        self
    }

    pub fn with_vrm(mut self, vrm_asset: impl Into<String>) -> Self {
        self.vrm_asset = Some(vrm_asset.into());
        self
    }

    pub fn with_camera_active(mut self, active: bool) -> Self {
        self.camera_active = Some(active);
        self
    }

    pub fn spawn(&self, commands: &mut Commands, asset_server: &AssetServer) -> Entity {
        let config = self.config.clone().unwrap_or_default();
        let tracking_source = self.tracking_source.unwrap_or_default();

        let camera = commands
            .spawn((
                PlayerCamera,
                Camera {
                    is_active: self.camera_active.unwrap_or(true),
                    ..default()
                },
                Hdr,
                Atmosphere::EARTH,
                AtmosphereSettings::default(),
                AutoExposure {
                    range: -3.0..=6.0,
                    ..Default::default()
                },
                Exposure::SUNLIGHT,
                Bloom::OLD_SCHOOL,
                Transform::default().looking_at(Vec3::NEG_Z, Vec3::Y),
                RenderLayers::layer(0).union(&RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly]),
            ))
            .id();

        let player_rig = commands
            .spawn((
                PlayerRig,
                RigidBody::Dynamic,
                Collider::capsule(PLAYER_RADIUS, config.real_height),
                TnuaController::default(),
                TnuaAvian3dSensorShape(Collider::cylinder(PLAYER_RADIUS - 0.01, 0.0)),
                LockedAxes::ROTATION_LOCKED,
                Transform::from_xyz(0.0, config.real_height / 2.0, 0.0),
            ))
            .id();

        let initial_eye_y = config.real_height / 2.0 - 0.1;
        let tracked_head = commands
            .spawn((
                TrackedHead,
                TrackedPose::new(Vec3::new(0.0, initial_eye_y, 0.0), Quat::IDENTITY),
                Transform::from_xyz(0.0, initial_eye_y, 0.0),
            ))
            .add_child(camera)
            .id();

        let vrm_path = self
            .vrm_asset
            .as_deref()
            .unwrap_or(DEFAULT_AVATAR)
            .to_string();
        let vrm_handle = asset_server.load(vrm_path);
        let animations = default_character_animations(asset_server);

        let avatar = commands
            .spawn((
                PlayerAvatar,
                AvatarBones::default(),
                AverageVelocity {
                    target: Some(player_rig),
                    ..Default::default()
                },
                VrmBundle {
                    vrm: VrmInstance(vrm_handle),
                    ..Default::default()
                },
                Transform::from_xyz(0.0, -config.real_height / 2.0, 0.0),
                animations,
            ))
            .id();

        commands
            .entity(player_rig)
            .add_children(&[avatar, tracked_head]);

        commands
            .spawn((
                LocalPlayer,
                PlayerEntities {
                    avatar,
                    camera,
                    rig: player_rig,
                    tracked_head,
                },
                config,
                tracking_source,
                Transform::default(),
            ))
            .add_child(player_rig)
            .id()
    }
}
