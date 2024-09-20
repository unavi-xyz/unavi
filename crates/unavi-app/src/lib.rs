//! The core UNAVI app, built with [Bevy](https://bevyengine.org/).
//!
//! # Building
//!
//! A few additional tools are required to build the `unavi-app` crate.
//! These are handled automatically if using the Nix flake, or they can be installed manually if you are not so based.
//!
//! ## Cargo
//!
//! The `cargo-component` and `wac-cli` cargo tools are used to build and compose WASM components.
//!
//! ```bash
//! cargo install cargo-component
//! cargo install wac-cli
//! ```
//!
//! ## Cap'n Proto
//!
//! [Cap'n Proto](https://capnproto.org/install.html) must be installed to compile networking schemas.
//!
//! ## Git Submodules
//!
//! Make sure your Git submodules are up to date:
//!
//! ```bash
//! git submodule update --init --recursive
//! git submodule foreach git pull
//! ```

use std::sync::Arc;

use bevy::{
    asset::AssetMetaCheck,
    log::{Level, LogPlugin},
    prelude::*,
    utils::HashSet,
};

use avian3d::prelude::*;
use dwn::{actor::Actor, store::SurrealStore, DWN};
use surrealdb::{engine::local::Db, Surreal};
use unavi_world::UserActor;

#[cfg(not(target_family = "wasm"))]
pub mod native;
mod unavi_system;

pub struct StartOptions {
    pub debug_physics: bool,
    pub log_level: Level,
    pub xr: bool,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            debug_physics: false,
            log_level: Level::INFO,
            xr: false,
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

    let mut app = App::new();

    let default_plugins = if opts.xr {
        #[cfg(target_family = "wasm")]
        {
            DefaultPlugins.build()
        }
        #[cfg(not(target_family = "wasm"))]
        {
            bevy_oxr::DefaultXrPlugins::default().build()
        }
    } else {
        DefaultPlugins.build()
    };

    default_plugins
        .set(AssetPlugin {
            meta_check: AssetMetaCheck::Paths(meta_paths),
            ..default()
        })
        .set(LogPlugin {
            level: opts.log_level,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                prevent_default_event_handling: true,
                title: "UNAVI".to_string(),
                ..default()
            }),
            ..default()
        })
        .finish(&mut app);

    app.insert_resource(UserActor(actor)).add_plugins((
        PhysicsPlugins::default(),
        unavi_networking::NetworkingPlugin,
        unavi_player::PlayerPlugin,
        unavi_scripting::ScriptingPlugin,
        unavi_settings::SettingsPlugin,
        unavi_world::WorldPlugin,
    ));

    app.add_systems(Startup, unavi_system::spawn_unavi_system);
    #[cfg(not(target_family = "wasm"))]
    app.add_systems(Startup, native::icon::set_window_icon);

    if opts.debug_physics {
        app.add_plugins(PhysicsDebugPlugin::default());
    }

    app.run();
}
