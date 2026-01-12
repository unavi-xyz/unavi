#![allow(dead_code)]

mod did_key;
mod did_web;

use std::{fmt::Debug, sync::Arc};

use iroh::{Endpoint, RelayMode};
use rstest::fixture;
use rusqlite::params;
use wds::{
    DataStore,
    actor::Actor,
    record::{
        acl::Acl,
        schema::{SCHEMA_ACL, SCHEMA_RECORD},
    },
};
use xdid::core::did::Did;

use did_key::{generate_actor, generate_actor_with_identity};
use did_web::{DidWebServer, generate_actor_web};

pub struct DataStoreCtx {
    pub store: DataStore,
    pub alice: Actor,
    pub bob: Actor,
}

#[fixture]
pub async fn ctx() -> DataStoreCtx {
    let endpoint = Endpoint::empty_builder(RelayMode::Disabled)
        .bind()
        .await
        .expect("bind endpoint");

    let store = DataStore::builder(endpoint)
        .build()
        .await
        .expect("construct data store");

    // Upload schemas to the blob store.
    for schema in [&SCHEMA_ACL, &SCHEMA_RECORD] {
        store
            .blobs()
            .blobs()
            .add_slice(&schema.bytes)
            .await
            .expect("add schema");
    }

    let alice = generate_actor(&store).await;
    let bob = generate_actor(&store).await;

    DataStoreCtx { store, alice, bob }
}

pub struct MultiStoreCtx {
    pub rome: DataStoreCtx,
    pub carthage: DataStoreCtx,
    _alice_server: DidWebServer,
    _bob_server: DidWebServer,
}

/// Multi-store context using did:web with DID document service auth.
#[fixture]
pub async fn multi_ctx() -> MultiStoreCtx {
    let mut rome = ctx().await;
    let mut carthage = ctx().await;

    // Both WDS endpoints can authenticate on behalf of these actors.
    let wds_endpoints = vec![rome.store.endpoint().id(), carthage.store.endpoint().id()];

    // Generate did:web actors shared across both stores.
    let alice_with_server = generate_actor_web(&rome.store, wds_endpoints.clone()).await;
    let bob_with_server = generate_actor_web(&rome.store, wds_endpoints).await;

    // Set up quotas on carthage for these actors.
    for did in [
        alice_with_server.actor.identity().did(),
        bob_with_server.actor.identity().did(),
    ] {
        let did_str = did.to_string();
        carthage
            .store
            .db()
            .async_call(move |conn| {
                conn.execute(
                    "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
                    params![&did_str],
                )?;
                Ok(())
            })
            .await
            .expect("create quota on carthage");
    }

    // Replace actors with did:web versions.
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
    pub bob_ctx: DataStoreCtx,
}

/// Multi-store context using did:key with `set_user_identity` for sync auth.
/// Tests the embedded WDS authentication pattern.
#[fixture]
pub async fn multi_ctx_local() -> LocalStoreCtx {
    let mut alice_ctx = ctx().await;
    let mut bob_ctx = ctx().await;

    // Generate did:key identities for each user.
    let (_, alice_identity) = generate_actor_with_identity(&alice_ctx.store).await;
    let (_, bob_identity) = generate_actor_with_identity(&bob_ctx.store).await;

    // Set user identities on respective stores for sync authentication.
    alice_ctx
        .store
        .set_user_identity(Arc::clone(&alice_identity));
    bob_ctx.store.set_user_identity(Arc::clone(&bob_identity));

    // Set up quotas on both stores for both users.
    for (store, did) in [
        (&alice_ctx.store, alice_identity.did()),
        (&alice_ctx.store, bob_identity.did()),
        (&bob_ctx.store, alice_identity.did()),
        (&bob_ctx.store, bob_identity.did()),
    ] {
        let did_str = did.to_string();
        store
            .db()
            .async_call(move |conn| {
                conn.execute(
                    "INSERT OR IGNORE INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
                    params![&did_str],
                )?;
                Ok(())
            })
            .await
            .expect("create quota");
    }

    // Replace actors with their respective identities.
    alice_ctx.alice = alice_ctx.store.local_actor(Arc::clone(&alice_identity));
    alice_ctx.bob = alice_ctx.store.local_actor(Arc::clone(&bob_identity));
    bob_ctx.alice = bob_ctx.store.local_actor(Arc::clone(&alice_identity));
    bob_ctx.bob = bob_ctx.store.local_actor(Arc::clone(&bob_identity));

    LocalStoreCtx { alice_ctx, bob_ctx }
}

pub fn assert_contains(e: impl Debug, contains: &str) {
    let e = format!("{e:?}");
    assert!(e.contains(contains), "'{e}' does not contain '{contains}'");
}

/// Creates a default ACL with only the given DID having full access.
pub fn acl_only(did: &Did) -> Acl {
    Acl {
        public: false,
        manage: vec![did.clone()],
        write: vec![did.clone()],
        read: vec![],
    }
}

/// Creates an ACL with manage for owner and write for writer.
pub fn acl_with_writer(owner: &Did, writer: &Did) -> Acl {
    Acl {
        public: false,
        manage: vec![owner.clone()],
        write: vec![owner.clone(), writer.clone()],
        read: vec![],
    }
}

/// Creates an ACL with manage for owner and read for reader.
pub fn acl_with_reader(owner: &Did, reader: &Did) -> Acl {
    Acl {
        public: false,
        manage: vec![owner.clone()],
        write: vec![owner.clone()],
        read: vec![reader.clone()],
    }
}
