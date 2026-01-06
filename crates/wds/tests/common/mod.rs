#![allow(dead_code)]

mod did_key;
mod did_web;

use std::{fmt::Debug, sync::Arc};

use iroh::{Endpoint, RelayMode};
use rstest::fixture;
use tempfile::{TempDir, tempdir};
use wds::{
    DataStore,
    actor::Actor,
    record::{
        acl::Acl,
        schema::{SCHEMA_ACL, SCHEMA_RECORD},
    },
};
use xdid::core::did::Did;

use did_key::generate_actor;
use did_web::{DidWebServer, generate_actor_web};

pub struct DataStoreCtx {
    pub store: DataStore,
    pub alice: Actor,
    pub bob: Actor,
    _dir: TempDir,
}

#[fixture]
pub async fn ctx() -> DataStoreCtx {
    let dir = tempdir().expect("tempdir");

    let endpoint = Endpoint::empty_builder(RelayMode::Disabled)
        .bind()
        .await
        .expect("bind endpoint");

    let store = DataStore::new(dir.path(), endpoint)
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

    DataStoreCtx {
        store,
        alice,
        bob,
        _dir: dir,
    }
}

pub struct MultiStoreCtx {
    pub rome: DataStoreCtx,
    pub carthage: DataStoreCtx,
    alice_server: DidWebServer,
    bob_server: DidWebServer,
}

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
        sqlx::query!(
            "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
            did_str
        )
        .execute(carthage.store.db())
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
        alice_server: alice_with_server.server,
        bob_server: bob_with_server.server,
    }
}

pub fn assert_contains(e: impl Debug, contains: &str) {
    let e = format!("{e:?}");
    assert!(e.contains(contains), "'{e}' does not contain '{contains}'");
}

/// Creates a default ACL with only the given DID having full access.
pub fn acl_only(did: &Did) -> Acl {
    Acl {
        manage: vec![did.clone()],
        write: vec![did.clone()],
        read: vec![],
    }
}

/// Creates an ACL with manage for owner and write for writer.
pub fn acl_with_writer(owner: &Did, writer: &Did) -> Acl {
    Acl {
        manage: vec![owner.clone()],
        write: vec![owner.clone(), writer.clone()],
        read: vec![],
    }
}

/// Creates an ACL with manage for owner and read for reader.
pub fn acl_with_reader(owner: &Did, reader: &Did) -> Acl {
    Acl {
        manage: vec![owner.clone()],
        write: vec![owner.clone()],
        read: vec![reader.clone()],
    }
}
