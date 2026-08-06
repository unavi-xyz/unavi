#![allow(dead_code)]

mod did_key;
mod did_web;

use std::{
    fmt::Display,
    sync::Arc,
};

use did_key::{
    generate_actor,
    generate_actor_with_identity,
};
use did_web::{
    DidWebServer,
    generate_actor_web,
};
use iroh::{
    Endpoint,
    endpoint::presets::N0DisableRelay,
    protocol::Router,
};
use rstest::fixture;
use rusqlite::params;
use wds::{
    DataStore,
    actor::Actor,
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

    let (store, f) = DataStore::builder(endpoint.clone())
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

pub struct MultiStoreCtx {
    pub rome:      DataStoreCtx,
    pub carthage:  DataStoreCtx,
    _alice_server: DidWebServer,
    _bob_server:   DidWebServer,
}

/// Multi-store context using did:web with DID document service auth.
#[fixture]
pub async fn multi_ctx() -> MultiStoreCtx {
    let mut rome = ctx().await;
    let mut carthage = ctx().await;

    // Both WDS endpoints can authenticate on behalf of these actors.
    let wds_endpoints = vec![rome.store.endpoint().id(), carthage.store.endpoint().id()];

    let alice_with_server = generate_actor_web(&rome.store, wds_endpoints.clone()).await;
    let bob_with_server = generate_actor_web(&rome.store, wds_endpoints).await;

    for did in [
        alice_with_server.actor.identity().did(),
        bob_with_server.actor.identity().did(),
    ] {
        let did_str = did.to_string();
        carthage
            .store
            .db()
            .call(move |conn| {
                conn.execute(
               "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
               params![&did_str],
            )?;
                Ok(())
            })
            .await
            .expect("create quota on carthage");
    }

    rome.alice = rome
        .store
        .local_actor(Arc::clone(alice_with_server.actor.identity()));
    rome.bob = rome
        .store
        .local_actor(Arc::clone(bob_with_server.actor.identity()));
    carthage.alice = carthage
        .store
        .local_actor(Arc::clone(alice_with_server.actor.identity()));
    carthage.bob = carthage
        .store
        .local_actor(Arc::clone(bob_with_server.actor.identity()));

    MultiStoreCtx {
        rome,
        carthage,
        _alice_server: alice_with_server.server,
        _bob_server: bob_with_server.server,
    }
}

/// Two stores with user identities set for embedded WDS sync auth pattern.
pub struct LocalStoreCtx {
    /// Alice's store with her identity set.
    pub alice_ctx: DataStoreCtx,
    /// Bob's store with his identity set.
    pub bob_ctx:   DataStoreCtx,
}

/// Multi-store context using did:key with `set_user_identity` for sync auth.
/// Tests the embedded WDS authentication pattern.
#[fixture]
pub async fn multi_ctx_local() -> LocalStoreCtx {
    let mut alice_ctx = ctx().await;
    let mut bob_ctx = ctx().await;

    let (_, alice_identity) = generate_actor_with_identity(&alice_ctx.store).await;
    let (_, bob_identity) = generate_actor_with_identity(&bob_ctx.store).await;

    alice_ctx
        .store
        .set_user_identity(Arc::clone(&alice_identity));
    bob_ctx.store.set_user_identity(Arc::clone(&bob_identity));

    for (store, did) in [
        (&alice_ctx.store, alice_identity.did()),
        (&alice_ctx.store, bob_identity.did()),
        (&bob_ctx.store, alice_identity.did()),
        (&bob_ctx.store, bob_identity.did()),
    ] {
        let did_str = did.to_string();
        store
            .db()
            .call(move |conn| {
                conn.execute(
                    "INSERT OR IGNORE INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
                    params![&did_str],
                )?;
                Ok(())
            })
            .await
            .expect("create quota");
    }

    alice_ctx.alice = alice_ctx.store.local_actor(Arc::clone(&alice_identity));
    alice_ctx.bob = alice_ctx.store.local_actor(Arc::clone(&bob_identity));
    bob_ctx.alice = bob_ctx.store.local_actor(Arc::clone(&alice_identity));
    bob_ctx.bob = bob_ctx.store.local_actor(Arc::clone(&bob_identity));

    LocalStoreCtx { alice_ctx, bob_ctx }
}

pub fn assert_contains(err: impl Display, contains: &str) {
    let err = err.to_string();
    assert!(
        err.contains(contains),
        "'{err}' does not contain '{contains}'"
    );
}
