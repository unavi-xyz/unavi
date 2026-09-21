use std::time::Duration;

use bevy::prelude::*;
use unavi_physics::PhysicsPlugin;

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        bevy::scene::ScenePlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        PhysicsPlugin,
    ))
    .init_asset::<Mesh>()
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f32(1.0 / 60.0),
    ));
    app.finish();
    app.cleanup();
    app
}

pub fn step(app: &mut App, times: usize) {
    for _ in 0..times {
        app.update();
    }
}
