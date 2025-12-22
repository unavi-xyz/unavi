#![allow(dead_code)]

use rstest::fixture;
use tempfile::{TempDir, tempdir};
use wds::{DataStore, actor::Actor};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

pub struct DataStoreCtx {
    pub store: DataStore,
    pub alice: Actor,
    pub bob: Actor,
    _dir: TempDir,
}

fn generate_actor(store: &DataStore) -> Actor {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    store.actor(did, key)
}

#[fixture]
pub async fn ctx() -> DataStoreCtx {
    let dir = tempdir().expect("tempdir");
    let store = DataStore::new(dir.path())
        .await
        .expect("construct data store");

    let alice = generate_actor(&store);
    let bob = generate_actor(&store);

    DataStoreCtx {
        store,
        alice,
        bob,
        _dir: dir,
    }
}
