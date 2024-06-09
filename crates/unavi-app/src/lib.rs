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
use unavi_dwn::UserActor;

pub const ROOT_DIR: &str = ".unavi/app";

mod unavi_system;
#[cfg(feature = "self_update")]
#[cfg(not(target_family = "wasm"))]
pub mod update;

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
    let actor = Actor::new_did_key(dwn).expect("Failed to create DWN actor.");

    let mut meta_paths = HashSet::new();
    meta_paths.insert("images/dev-white.png".into());
    meta_paths.insert("images/skybox.png".into());

    let mut app = App::new();

    app.insert_resource(AssetMetaCheck::Paths(meta_paths))
        .insert_resource(UserActor(actor))
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: opts.log_level,
                ..default()
            }),
            PhysicsPlugins::default(),
            unavi_dwn::DwnPlugin,
            unavi_networking::NetworkingPlugin,
            unavi_player::PlayerPlugin,
            unavi_scripting::ScriptingPlugin,
            unavi_settings::SettingsPlugin,
            unavi_world::WorldPlugin,
        ))
        .add_systems(Startup, unavi_system::spawn_unavi_system);

    if opts.debug_physics {
        app.add_plugins(PhysicsDebugPlugin::default());
    }

    app.run();
}
