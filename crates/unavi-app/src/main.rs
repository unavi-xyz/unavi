#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use bevy::{asset::AssetMetaCheck, prelude::*, utils::HashSet};

use dwn::{actor::Actor, store::SurrealStore, DWN};
use surrealdb::{engine::local::Db, Surreal};
use unavi_app::{did::UserActor, UnaviPlugin};

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn wasm_start() {
    let db = Surreal::new::<surrealdb::engine::local::IndxDb>("unavi")
        .await
        .expect("Failed to create SurrealDB.");

    start(db).await
}

#[cfg(target_family = "wasm")]
fn main() {}

#[cfg(not(target_family = "wasm"))]
#[tokio::main]
async fn main() {
    const DB_PATH: &str = ".unavi/app-db";

    std::fs::create_dir_all(DB_PATH).expect("Failed to create database dir.");

    let db = Surreal::new::<surrealdb::engine::local::SpeeDb>(DB_PATH)
        .await
        .expect("Failed to create SurrealDB.");

    start(db).await
}

async fn start(db: Surreal<Db>) {
    let store = SurrealStore::new(db)
        .await
        .expect("Failed to create DWN store.");
    let dwn = Arc::new(DWN::from(store));
    let actor = Actor::new_did_key(dwn).expect("Failed to create DWN actor.");

    let mut meta_paths = HashSet::new();
    meta_paths.insert("images/dev-white.png".into());
    meta_paths.insert("images/skybox-1-512.png".into());

    App::new()
        .insert_resource(UserActor(actor))
        .insert_resource(AssetMetaCheck::Paths(meta_paths))
        .add_plugins((DefaultPlugins, UnaviPlugin::default()))
        .run();
}
