//! Rendering settings applied to every camera that spawns, plus the sky
//! material.

use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::{
        Exposure,
        Hdr,
    },
    light::ShadowFilteringMethod,
    post_process::bloom::Bloom,
    prelude::*,
};
use unavi_portal::view::camera::PortalViewCamera;

mod sky;

const FOG_END: f32 = 1000.0;
const FOG_START: f32 = FOG_END * 0.8;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<sky::SkyMaterial>::default())
            .init_resource::<sky::blend::SkyTarget>()
            .init_resource::<sky::blend::SkyBlend>()
            .add_systems(Startup, sky::spawn_sky)
            .add_systems(
                Update,
                (sky::blend::target_sky, sky::blend::blend_sky).chain(),
            )
            .add_systems(FixedUpdate, apply_camera_effects);
    }
}

pub fn apply_camera_effects(
    mut commands: Commands,
    new_cameras: Query<Entity, (Added<Camera3d>, Without<PortalViewCamera>)>,
    sky: Res<sky::blend::SkyBlend>,
) {
    for entity in new_cameras {
        commands.entity(entity).insert((
            Hdr,
            Bloom::OLD_SCHOOL,
            Exposure::SUNLIGHT,
            Msaa::Off,
            ShadowFilteringMethod::Temporal,
            TemporalAntiAliasing::default(),
            DistanceFog {
                color: sky.palette().fog().into(),
                falloff: FogFalloff::Linear {
                    start: FOG_START,
                    end:   FOG_END,
                },
                ..default()
            },
        ));
    }
}
