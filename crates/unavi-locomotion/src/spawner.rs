use avian3d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::{
    camera::{Exposure, visibility::RenderLayers},
    pbr::{Atmosphere, AtmosphereSettings},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_vrm::first_person::{DEFAULT_RENDER_LAYERS, FirstPersonFlag};
use unavi_avatar::{AvatarSpawner, AverageVelocity, default_character_animations};
use unavi_portal::{PortalTraveler, create::PORTAL_RENDER_LAYER};

use crate::{
    AgentCamera, AgentEntities, AgentRig, ControlScheme, ControlSchemeConfig, Grounded, LocalAgent,
    config::AgentConfig,
    tracking::{TrackedHead, TrackedPose, TrackingSource},
};

/// Builder for spawning a local agent entity.
#[derive(Default)]
pub struct LocalAgentSpawner {
    pub config: Option<AgentConfig>,
    pub tracking_source: Option<TrackingSource>,
    pub vrm_asset: Option<String>,
}

pub struct SpawnedAgent {
    pub body: Entity,
    pub camera: Entity,
    pub root: Entity,
}

impl LocalAgentSpawner {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_config(mut self, config: AgentConfig) -> Self {
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

    pub fn spawn(&self, commands: &mut Commands, asset_server: &AssetServer) -> SpawnedAgent {
        let config = self.config.clone().unwrap_or_default();
        let tracking_source = self.tracking_source.unwrap_or_default();

        let camera = spawn_camera(commands, asset_server);

        let body = commands
            .spawn((
                AgentRig,
                Grounded(true),
                RigidBody::Dynamic,
                Pickable::IGNORE,
                Collider::capsule(config.effective_vrm_radius(), config.effective_vrm_height()),
                TnuaController::<ControlScheme>::default(),
                TnuaConfig::<ControlScheme>(asset_server.add(ControlSchemeConfig {
                    basis: TnuaBuiltinWalkConfig {
                        float_height: config.float_height(),
                        max_slope: 55_f32.to_radians(),
                        ..Default::default()
                    },
                    jump: TnuaBuiltinJumpConfig {
                        height: config.jump_height,
                        ..Default::default()
                    },
                })),
                TnuaAvian3dSensorShape(Collider::cylinder(
                    config.effective_vrm_radius() - 0.01,
                    0.0,
                )),
                LockedAxes::ROTATION_LOCKED,
                Transform::from_xyz(0.0, config.effective_vrm_height() / 2.0, 0.0),
                PortalTraveler,
            ))
            .id();

        let initial_eye_y = config.effective_vrm_height() / 2.0 - 0.1;
        let tracked_head = commands
            .spawn((
                TrackedHead,
                TrackedPose::new(Vec3::new(0.0, initial_eye_y, 0.0), Quat::IDENTITY),
                Transform::from_xyz(0.0, initial_eye_y, 0.0),
            ))
            .add_child(camera)
            .id();

        let avatar = AvatarSpawner {
            vrm_asset: self.vrm_asset.clone(),
        }
        .spawn(commands, asset_server);

        let animations = default_character_animations(asset_server);
        commands.entity(avatar).insert((
            AverageVelocity {
                target: Some(body),
                ..Default::default()
            },
            animations,
        ));

        commands.entity(avatar).insert(Transform::from_xyz(
            0.0,
            -config.effective_vrm_height() / 2.0,
            0.0,
        ));

        commands.entity(body).add_children(&[avatar, tracked_head]);

        let root = commands
            .spawn((
                LocalAgent,
                AgentEntities {
                    avatar,
                    camera,
                    body,
                    tracked_head,
                },
                config,
                tracking_source,
                Transform::default(),
                Visibility::default(),
            ))
            .add_child(body)
            .id();

        SpawnedAgent { body, camera, root }
    }
}

fn spawn_camera(commands: &mut Commands, #[allow(unused)] asset_server: &AssetServer) -> Entity {
    let fog_color = Color::Srgba(Srgba::from_u8_array([0, 192, 240, 255]));
    let fog_end = 1000.0;

    let camera = commands
        .spawn((
            AgentCamera,
            Camera::default(),
            Projection::Perspective(PerspectiveProjection {
                near: 0.001,
                ..default()
            }),
            Hdr,
            Exposure::SUNLIGHT,
            Bloom::OLD_SCHOOL,
            Msaa::Sample4,
            Atmosphere::EARTH,
            AtmosphereSettings::default(),
            Transform::default().looking_at(Vec3::NEG_Z, Vec3::Y),
            RenderLayers::from_layers(&[0, PORTAL_RENDER_LAYER])
                .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly]),
            DistanceFog {
                color: fog_color,
                falloff: FogFalloff::Linear {
                    start: fog_end * 0.8,
                    end: fog_end,
                },
                ..default()
            },
        ))
        .id();

    #[cfg(all(target_family = "wasm", not(feature = "webgpu")))]
    commands.entity(camera).insert((
        // https://github.com/bevyengine/bevy/issues/14340
        bevy::post_process::auto_exposure::AutoExposure {
            range: -4.0..=4.0,
            ..default()
        },
        // Atmospheric shader does not support WebGL, so add a skybox.
        Mesh3d(asset_server.add(Cuboid::from_size(Vec3::splat(fog_end)).mesh().build())),
        MeshMaterial3d(asset_server.add(StandardMaterial {
            base_color: fog_color,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
    ));

    camera
}
