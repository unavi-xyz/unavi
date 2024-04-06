#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use bevy::{asset::AssetMetaCheck, prelude::*, utils::HashSet};

use dwn::{actor::Actor, store::SurrealStore, DWN};
use surrealdb::{engine::local::SpeeDb, Surreal};
use unavi_app::{did::User, UnaviPlugin};

const DB_PATH: &str = ".unavi/app-db";

#[tokio::main]
async fn main() {
    std::fs::create_dir_all(DB_PATH).expect("Failed to create database dir.");

    let db = Surreal::new::<SpeeDb>(DB_PATH)
        .await
        .expect("Failed to create SurrealDB.");
    let store = SurrealStore::new(db)
        .await
        .expect("Failed to create DWN store.");
    let dwn = Arc::new(DWN::from(store));
    let actor = Actor::new_did_key(dwn).expect("Failed to create DWN actor.");

    let mut meta_paths = HashSet::new();
    meta_paths.insert("images/dev-white.png".into());
    meta_paths.insert("images/skybox-1-512.png".into());

    App::new()
        .insert_resource(User { actor })
        .insert_resource(AssetMetaCheck::Paths(meta_paths))
        .add_plugins((DefaultPlugins, UnaviPlugin::default()))
        .run();
}
