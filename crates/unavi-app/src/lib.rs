//! The core UNAVI app, built with [Bevy](https://bevyengine.org/).

use std::sync::Arc;

use bevy::{
    asset::AssetMetaCheck,
    log::{Level, LogPlugin},
    prelude::*,
    utils::HashSet,
};

use bevy_xpbd_3d::plugins::{PhysicsDebugPlugin, PhysicsPlugins};
use dwn::{actor::Actor, store::SurrealStore, DWN};
use surrealdb::{engine::local::Db, Surreal};

pub struct StartOptions {
    pub debug_physics: bool,
    pub log_level: Level,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            debug_physics: false,
            log_level: Level::INFO,
        }
    }
}

pub async fn start(db: Surreal<Db>, opts: StartOptions) {
    let store = SurrealStore::new(db)
        .await
        .expect("Failed to create DWN store.");

    let dwn = Arc::new(DWN::from(store));
    let _actor = Actor::new_did_key(dwn).expect("Failed to create DWN actor.");

    let mut meta_paths = HashSet::new();
    meta_paths.insert("images/dev-white.png".into());
    meta_paths.insert("images/skybox-1-512.png".into());

    let mut app = App::new();

    app.insert_resource(AssetMetaCheck::Paths(meta_paths))
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: opts.log_level,
                ..default()
            }),
            PhysicsPlugins::default(),
            // unavi_did::DidPlugin,
            // unavi_networking::NetworkingPlugin,
            unavi_player::PlayerPlugin,
            // unavi_scripting::ScriptingPlugin,
            unavi_settings::SettingsPlugin,
            unavi_world::WorldPlugin,
        ));

    if opts.debug_physics {
        app.add_plugins(PhysicsDebugPlugin::default());
    }

    app.run();
}
