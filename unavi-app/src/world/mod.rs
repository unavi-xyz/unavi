use bevy::prelude::*;

mod skybox;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_world, skybox::create_skybox))
            .add_systems(
                Update,
                (skybox::add_skybox_to_cameras, skybox::process_cubemap),
            );
    }
}

fn setup_world(mut commands: Commands, mut ambient: ResMut<AmbientLight>) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 10_000.0,
                color: Color::rgb(1.0, 1.0, 0.98),
                ..default()
            },
            transform: Transform::from_xyz(-4.5, 10.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        bevy_vrm::mtoon::MtoonSun,
    ));

    ambient.color = Color::rgb(0.95, 0.95, 1.0);
    ambient.brightness = 0.1;
}
