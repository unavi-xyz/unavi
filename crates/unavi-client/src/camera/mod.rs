use bevy::{camera::Exposure, post_process::bloom::Bloom, prelude::*, render::view::Hdr};

mod sky;

const FOG_COLOR: Color = Color::Srgba(Srgba::new(0.7, 0.7, 0.7, 0.7));
const FOG_END: f32 = 1000.0;
const FOG_START: f32 = FOG_END * 0.8;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MaterialPlugin::<sky::SkyMaterial>::default(),))
            .add_systems(Startup, sky::spawn_sky)
            .add_systems(FixedUpdate, apply_camera_effects);
    }
}

pub fn apply_camera_effects(mut commands: Commands, new_cameras: Query<Entity, Added<Camera3d>>) {
    for entity in new_cameras {
        commands.entity(entity).insert((
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
        ));
    }
}
