// Compiled into every integration-test binary in this crate, each of which
// only uses a subset of these helpers.
#![expect(dead_code)]

mod did_key;
pub mod did_web;

use did_key::generate_actor;
use iroh::{
    Endpoint,
    endpoint::presets::N0DisableRelay,
    protocol::Router,
};
use rstest::fixture;
use wds::{
    DataStore,
    actor::Actor,
    identity::{
        WdsIdentity,
        store::KeyStorage,
    },
};

pub struct DataStoreCtx {
    pub store: DataStore,
    pub alice: Actor,
    pub bob:   Actor,
    router:    Router,
}

#[fixture]
pub async fn ctx() -> DataStoreCtx {
    let endpoint = Endpoint::builder(N0DisableRelay)
        .bind()
        .await
        .expect("bind endpoint");

    let identity = std::sync::Arc::new(
        WdsIdentity::load(&KeyStorage::Ephemeral).expect("generate host identity"),
    );

    let (store, f) = DataStore::builder(endpoint.clone(), identity)
        .build()
        .await
        .expect("construct data store");

    let rb = Router::builder(endpoint);
    let rb = f(rb);
    let router = rb.spawn();

    let alice = generate_actor(&store).await;
    let bob = generate_actor(&store).await;

    DataStoreCtx {
        store,
        alice,
        bob,
        router,
    }
}
