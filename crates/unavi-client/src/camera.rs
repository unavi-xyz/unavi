use bevy::{camera::Exposure, post_process::bloom::Bloom, prelude::*, render::view::Hdr};

const FOG_COLOR: Color = Color::Srgba(Srgba::new(0.0, 0.75, 0.95, 1.0));
const FOG_END: f32 = 1000.0;
const FOG_START: f32 = FOG_END * 0.8;

#[derive(EntityEvent)]
pub struct ApplyCameraEffects {
    entity: Entity,
}

pub fn apply_camera_effects(mut commands: Commands, new_cameras: Query<Entity, Added<Camera3d>>) {
    for entity in new_cameras {
        commands
            .entity(entity)
            .insert((
                Hdr,
                Exposure::SUNLIGHT,
                Bloom::OLD_SCHOOL,
                Msaa::Sample4,
                DistanceFog {
                    color: FOG_COLOR,
                    falloff: FogFalloff::Linear {
                        start: FOG_START,
                        end: FOG_END,
                    },
                    ..default()
                },
            ))
            .trigger(|entity| ApplyCameraEffects { entity });
    }
}

/// WebGPU effects (runs in web or native)
#[cfg(not(all(target_family = "wasm", not(feature = "webgpu"))))]
pub fn on_apply_camera_effects(
    trigger: On<ApplyCameraEffects>,
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<bevy::pbr::ScatteringMedium>>,
) {
    use bevy::{
        pbr::{Atmosphere, AtmosphereSettings},
        post_process::auto_exposure::AutoExposure,
    };

    commands.entity(trigger.entity).insert((
        AutoExposure {
            range: -4.0..=4.0,
            ..default()
        },
        Atmosphere::earthlike(scattering_mediums.add(bevy::pbr::ScatteringMedium::default())),
        AtmosphereSettings::default(),
    ));
}

/// WebGL effects (only runs in web when webgpu is disabled)
#[cfg(all(target_family = "wasm", not(feature = "webgpu")))]
pub fn on_apply_camera_effects(
    trigger: On<ApplyCameraEffects>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // TODO webgl sky shader

    // Basic skybox attached to camera
    commands.entity(trigger.entity).insert((
        Mesh3d(asset_server.add(Cuboid::from_size(Vec3::splat(FOG_END)).mesh().build())),
        MeshMaterial3d(asset_server.add(StandardMaterial {
            base_color: FOG_COLOR,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
    ));
}
