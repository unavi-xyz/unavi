use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use unavi_scripting::{ScriptBundle, ScriptingPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "../unavi-app/assets".to_string(),
                ..Default::default()
            }),
            PanOrbitCameraPlugin,
            ScriptingPlugin,
        ))
        .add_systems(Startup, (setup_scene, load_script))
        .run();
}

fn setup_scene(mut ambient: ResMut<AmbientLight>, mut commands: Commands) {
    ambient.brightness = 100.0;
    ambient.color = Color::rgb(0.95, 0.95, 1.0);

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.5, 10.0, -7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let mut transform = Transform::from_xyz(0.0, 3.0, -10.0);
    transform.look_at(Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 1.0, 0.0));

    commands.spawn((
        Camera3dBundle {
            transform,
            ..Default::default()
        },
        PanOrbitCamera::default(),
    ));
}

const NAME: &str = "example_wired_gltf";

pub fn load_script(asset_server: Res<AssetServer>, mut commands: Commands) {
    let path = format!("components/{}_{}.wasm", NAME, env!("CARGO_PKG_VERSION"));
    let wasm = asset_server.load(path);

    commands.spawn((
        ScriptBundle {
            name: NAME.into(),
            wasm,
        },
        SpatialBundle::default(),
    ));
}
