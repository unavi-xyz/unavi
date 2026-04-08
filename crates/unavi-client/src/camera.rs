use bevy::{camera::Exposure, post_process::bloom::Bloom, prelude::*, render::view::Hdr};

pub fn apply_camera_effects(
    mut commands: Commands,
    new_cameras: Query<Entity, Added<Camera3d>>,
    #[cfg(not(all(target_family = "wasm", not(feature = "webgpu"))))]
    mut scattering_mediums: ResMut<Assets<bevy::pbr::ScatteringMedium>>,
    #[cfg(all(target_family = "wasm", not(feature = "webgpu")))] asset_server: Res<AssetServer>,
) {
    let fog_color = Color::Srgba(Srgba::from_u8_array([0, 192, 240, 255]));
    let fog_end = 1000.0;

    for entity in new_cameras {
        commands.entity(entity).insert((
            Hdr,
            Exposure::SUNLIGHT,
            Bloom::OLD_SCHOOL,
            Msaa::Sample4,
            DistanceFog {
                color: fog_color,
                falloff: FogFalloff::Linear {
                    start: fog_end * 0.8,
                    end: fog_end,
                },
                ..default()
            },
        ));

        #[cfg(not(all(target_family = "wasm", not(feature = "webgpu"))))]
        commands.entity(entity).insert((
            bevy::post_process::auto_exposure::AutoExposure {
                range: -4.0..=4.0,
                ..default()
            },
            bevy::pbr::Atmosphere::earthlike(
                scattering_mediums.add(bevy::pbr::ScatteringMedium::default()),
            ),
            bevy::pbr::AtmosphereSettings::default(),
        ));

        // No atmospheric shader in WebGL.
        #[cfg(all(target_family = "wasm", not(feature = "webgpu")))]
        {
            commands.entity(entity).insert((
                Mesh3d(asset_server.add(Cuboid::from_size(Vec3::splat(fog_end)).mesh().build())),
                MeshMaterial3d(asset_server.add(StandardMaterial {
                    base_color: fog_color,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })),
            ));
        }
    }
}
