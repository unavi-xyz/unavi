use avian3d::prelude::*;
use bevy::prelude::*;
use unavi_player::PlayerPlugin;
use unavi_scripting::{ScriptBundle, ScriptingPlugin};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            file_path: "../unavi-app/assets".to_string(),
            ..Default::default()
        }),
        PhysicsDebugPlugin::default(),
        PhysicsPlugins::default(),
        PlayerPlugin,
        ScriptingPlugin,
    ))
    .add_systems(Startup, (setup_scene, load_script))
    .run();
}

fn setup_scene(mut ambient: ResMut<AmbientLight>, mut commands: Commands) {
    ambient.brightness = 100.0;
    ambient.color = Color::linear_rgb(0.95, 0.95, 1.0);

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.5, 10.0, -7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(0.0, -1.0, 0.0),
            ..default()
        },
        Collider::cuboid(20.0, 0.5, 20.0),
        RigidBody::Static,
    ));
}

pub fn load_script(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((
        ScriptBundle::load("example:wired-player", &asset_server),
        SpatialBundle::default(),
    ));
}
