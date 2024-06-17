use bevy::prelude::*;
use unavi_scripting::ScriptBundle;

pub fn spawn_unavi_system(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((
        ScriptBundle::load("example:wired-gltf", &asset_server),
        SpatialBundle::default(),
    ));
}
