//! Rendering settings applied to every camera that spawns, plus the sky
//! material.

use bevy::{
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
pub mod veil;

const FOG_END: f32 = 1000.0;
const FOG_START: f32 = FOG_END * 0.8;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<sky::SkyMaterial>::default())
            .init_resource::<sky::blend::SkyTarget>()
            .init_resource::<sky::blend::SkyBlend>()
            .init_resource::<veil::Veil>()
            .insert_resource(ClearColor(
                sky::blend::SkyBlend::default().palette().fog().into(),
            ))
            .add_systems(Startup, sky::spawn_sky)
            .add_systems(
                Update,
                (
                    sky::blend::target_sky,
                    sky::blend::blend_sky,
                    veil::apply_veil,
                )
                    .chain(),
            )
            .add_observer(apply_camera_effects);
    }
}

pub fn apply_camera_effects(
    trigger: On<Add, Camera3d>,
    portal_views: Query<(), With<PortalViewCamera>>,
    mut commands: Commands,
    sky: Res<sky::blend::SkyBlend>,
    veil: Res<veil::Veil>,
) {
    let entity = trigger.entity;
    if portal_views.contains(entity) {
        return;
    }

    commands.entity(entity).insert((
        Hdr,
        Bloom::OLD_SCHOOL,
        Exposure::SUNLIGHT,
        ShadowFilteringMethod::Temporal,
        DistanceFog {
            color: sky.palette().fog().into(),
            falloff: veil.falloff(),
            ..default()
        },
    ));

    // TAA is unsupported on WebGL2, so fall back to MSAA.
    cfg_select! {
        all(target_family = "wasm", not(feature = "webgpu")) => {
            commands.entity(entity).insert(Msaa::Sample4);
        }
        _ => {
            commands.entity(entity).insert((
                Msaa::Off,
                bevy::anti_alias::taa::TemporalAntiAliasing::default(),
            ));
        }
    }
}
