use std::{future::Future, pin::Pin};

use surrealdb::{
    engine::local::{Db, SurrealKV},
    Error, Surreal,
};

use super::dirs::get_project_dirs;

pub fn get_filesystem_db() -> Pin<Box<dyn Future<Output = Result<Surreal<Db>, Error>>>> {
    Box::pin(async {
        let dirs = get_project_dirs();
        let db_path = dirs.data_dir();

        std::fs::create_dir_all(db_path).expect("Failed to create database dir.");

        let db = Surreal::new::<SurrealKV>(db_path).await;

        if let Err(e) = &db {
            if e.to_string().contains("given version is older") {
                // Breaking SurrealDB update.
                // Not sure how to properly migrate to a new version, so
                // delete all data and start fresh.
                std::fs::remove_dir_all(db_path).expect("Failed to remove database dir.");
                return get_filesystem_db().await;
            }
        }

        db
    })
}
