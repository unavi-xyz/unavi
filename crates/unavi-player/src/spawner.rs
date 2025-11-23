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
use bevy_vrm::first_person::{DEFAULT_RENDER_LAYERS, FirstPersonFlag};

use crate::{
    LocalPlayer, PlayerCamera, PlayerEntities, PlayerRig,
    animation::{defaults::default_character_animations, velocity::AverageVelocity},
    avatar_spawner::AvatarSpawner,
    config::{PLAYER_RADIUS, PlayerConfig},
    tracking::{TrackedHead, TrackedPose, TrackingSource},
};

/// Builder for spawning a local player entity.
#[derive(Default)]
pub struct LocalPlayerSpawner {
    pub config: Option<PlayerConfig>,
    pub tracking_source: Option<TrackingSource>,
    pub vrm_asset: Option<String>,
}

impl LocalPlayerSpawner {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_config(mut self, config: PlayerConfig) -> Self {
        self.config = Some(config);
        self
    }

    #[must_use]
    pub const fn with_tracking_source(mut self, source: TrackingSource) -> Self {
        self.tracking_source = Some(source);
        self
    }

    #[must_use]
    pub fn with_vrm(mut self, vrm_asset: impl Into<String>) -> Self {
        self.vrm_asset = Some(vrm_asset.into());
        self
    }

    pub fn spawn(&self, commands: &mut Commands, asset_server: &AssetServer) -> Entity {
        let config = self.config.clone().unwrap_or_default();
        let tracking_source = self.tracking_source.unwrap_or_default();

        let camera = commands
            .spawn((
                PlayerCamera,
                Camera::default(),
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
                RenderLayers::layer(0)
                    .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly]),
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
                Transform::from_xyz(0.0, config.real_height / 2.0 + 1.0, 0.0),
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

        // Spawn avatar using AvatarSpawner.
        let avatar = AvatarSpawner {
            vrm_asset: self.vrm_asset.clone(),
        }
        .spawn(commands, asset_server);

        // Add local-player-specific components.
        let animations = default_character_animations(asset_server);
        commands.entity(avatar).insert((
            AverageVelocity {
                target: Some(player_rig),
                ..Default::default()
            },
            animations,
        ));

        // Position avatar relative to rig.
        commands
            .entity(avatar)
            .insert(Transform::from_xyz(0.0, -config.real_height / 2.0, 0.0));

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
