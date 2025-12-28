#![allow(dead_code)]

use blake3::Hash;
use rstest::fixture;
use tempfile::{TempDir, tempdir};
use wds::{
    DataStore,
    actor::Actor,
    record::{
        acl::Acl,
        schema::{SCHEMA_STR_ACL, SCHEMA_STR_RECORD, Schema},
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
    let actor = store.actor(did.clone(), key);

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
    let store = DataStore::new_empty(dir.path())
        .await
        .expect("construct data store");

    // Upload built-in schemas to the blob store.
    for (_hash, bytes) in builtin_schema_bytes() {
        store
            .blobs()
            .blobs()
            .add_slice(&bytes)
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

#[must_use]
pub fn builtin_schema_bytes() -> Vec<(Hash, Vec<u8>)> {
    let acl_schema: Schema = ron::from_str(SCHEMA_STR_ACL).expect("valid acl schema");
    let record_schema: Schema = ron::from_str(SCHEMA_STR_RECORD).expect("valid record schema");

    vec![
        (
            acl_schema.id().expect("get acl schema id"),
            acl_schema.to_bytes().expect("serialize acl schema"),
        ),
        (
            record_schema.id().expect("get record schema id"),
            record_schema.to_bytes().expect("serialize record schema"),
        ),
    ]
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
