#![allow(dead_code)]

use std::{fmt::Debug, sync::Arc};

use iroh::{Endpoint, RelayMode};
use rstest::fixture;
use tempfile::{TempDir, tempdir};
use wds::{
    DataStore, Identity,
    actor::Actor,
    record::{
        acl::Acl,
        schema::{SCHEMA_ACL, SCHEMA_RECORD},
    },
};
use xdid::{
    core::did::Did,
    methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair},
};

pub struct DataStoreCtx {
    pub store: DataStore,
    pub alice: Actor,
    pub bob: Actor,
    _dir: TempDir,
}

async fn generate_actor(store: &DataStore) -> Actor {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    let identity = Arc::new(Identity::new(did.clone(), key));
    let actor = store.local_actor(identity);

    // Set up default quota for the actor.
    let did_str = did.to_string();
    sqlx::query!(
        "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
        did_str
    )
    .execute(store.db())
    .await
    .expect("create quota");

    actor
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
}

#[fixture]
pub async fn multi_ctx() -> MultiStoreCtx {
    let rome = ctx().await;
    let mut carthage = ctx().await;

    carthage.alice = carthage
        .store
        .local_actor(Arc::clone(rome.alice.identity()));
    carthage.bob = carthage.store.local_actor(Arc::clone(rome.bob.identity()));

    MultiStoreCtx { rome, carthage }
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
