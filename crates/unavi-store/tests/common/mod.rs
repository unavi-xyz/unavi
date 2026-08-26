// Compiled into every integration-test binary in this crate, each of which
// only uses a subset of these helpers.
#![allow(dead_code, reason = "each test binary uses one fixture")]

use std::time::Duration;

use iroh::{
    Endpoint,
    SecretKey,
    endpoint::presets::N0DisableRelay,
};
use iroh_docs::Author;
use rstest::fixture;
use unavi_store::builder::{
    Builder,
    Store,
};

#[fixture]
pub async fn store() -> Store {
    build(None).await
}

/// A store sweeping on `interval`, for tests that assert what garbage
/// collection reclaims.
pub async fn store_with_gc(interval: Duration) -> Store {
    build(Some(interval)).await
}

async fn build(gc: Option<Duration>) -> Store {
    let secret_key = SecretKey::generate();
    let author = Author::from_bytes(&secret_key.to_bytes());

    let endpoint = Endpoint::builder(N0DisableRelay)
        .secret_key(secret_key)
        .bind()
        .await
        .expect("bind endpoint");

    let builder = Builder::new(endpoint, author);
    let builder = match gc {
        Some(interval) => builder.gc_timer(interval),
        None => builder,
    };

    builder.build().await.expect("construct data store")
}
