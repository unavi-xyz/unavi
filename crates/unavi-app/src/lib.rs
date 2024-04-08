//! The core UNAVI app, built with [Bevy](https://bevyengine.org/).

use std::sync::Arc;

use bevy::{
    asset::AssetMetaCheck, diagnostic::FrameTimeDiagnosticsPlugin, log::LogPlugin, prelude::*,
    utils::HashSet,
};

pub mod avatar;
pub mod did;
pub mod networking;
pub mod player;
pub mod scripting;
pub mod settings;
pub mod state;
pub mod world;

pub use bevy::app::App;
use bevy_xpbd_3d::plugins::{PhysicsDebugPlugin, PhysicsPlugins};
use did::UserActor;
use dwn::{actor::Actor, store::SurrealStore, DWN};
use surrealdb::{engine::local::Db, Surreal};
use tracing::Level;

pub struct StartOptions {
    pub debug_frame_time: bool,
    pub debug_physics: bool,
    pub log_level: Level,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            debug_frame_time: false,
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
    meta_paths.insert("images/skybox-1-512.png".into());

    let mut app = App::new();

    app.insert_resource(UserActor(actor))
        .insert_resource(AssetMetaCheck::Paths(meta_paths))
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: opts.log_level,
                ..default()
            }),
            PhysicsPlugins::default(),
            avatar::AvatarPlugin,
            did::DidPlugin,
            networking::NetworkingPlugin,
            player::PlayerPlugin,
            scripting::ScriptingPlugin,
            settings::SettingsPlugin,
            world::WorldPlugin,
        ))
        .init_state::<state::AppState>();

    if opts.debug_physics {
        app.add_plugins(PhysicsDebugPlugin::default());
    }

    if opts.debug_frame_time {
        app.add_plugins(FrameTimeDiagnosticsPlugin);
    }

    app.run();
}
