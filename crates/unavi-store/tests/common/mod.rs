#![allow(dead_code, reason = "each test binary uses a subset of these")]

use std::{
    path::Path,
    time::Duration,
};

use iroh::{
    Endpoint,
    SecretKey,
    endpoint::presets::N0DisableRelay,
};
use iroh_docs::Author;
use rstest::fixture;
use unavi_store::{
    local::Storage,
    store::{
        Builder,
        Spawned,
    },
};

#[fixture]
pub async fn store() -> Spawned {
    build(|builder| builder).await
}

/// A store sweeping on `interval`, for tests that assert what garbage
/// collection reclaims.
pub async fn store_with_gc(interval: Duration) -> Spawned {
    build(move |builder| builder.gc_timer(interval)).await
}

/// A store whose recorded namespace ids land in `dir`, so a test can assert
/// what a restart would reopen.
pub async fn store_at(dir: &Path) -> Spawned {
    let dir = dir.to_path_buf();
    build(move |builder| builder.storage(Storage::Path(dir))).await
}

async fn build(configure: impl FnOnce(Builder) -> Builder) -> Spawned {
    let secret_key = SecretKey::generate();
    let author = Author::from_bytes(&secret_key.to_bytes());

    let endpoint = Endpoint::builder(N0DisableRelay)
        .secret_key(secret_key)
        .bind()
        .await
        .expect("bind endpoint");

    configure(Builder::new(endpoint, author))
        .build()
        .await
        .expect("construct data store")
}
