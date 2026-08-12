use std::{
    path::PathBuf,
    sync::Arc,
};

use iroh::{
    endpoint::presets::N0,
    protocol::Router,
};
use iroh_blobs::api::{
    Store,
    blobs::Blobs,
};
use iroh_docs::protocol::Docs;
use unavi_assets::MANIFEST;
use unavi_util::async_task::spawn_async_task;
use wds::{
    DataStore,
    actor::Actor,
    identity::Identity,
};
use xdid::methods::key::keys::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

pub struct TestWds {
    pub actor: Actor,
    pub blobs: Blobs,
    pub docs:  Docs,
    pub store: Store,
}

#[must_use]
pub fn create_test_wds() -> TestWds {
    let (tx, rx) = async_channel::bounded(1);

    spawn_async_task(async move {
        let endpoint = iroh::Endpoint::builder(N0)
            .bind()
            .await
            .expect("iroh endpoint");

        let (store, f) = DataStore::builder(endpoint.clone())
            .build()
            .await
            .expect("data store");

        let rb = Router::builder(endpoint);
        let rb = f(rb);
        let _router = rb.spawn();

        let blobs = store.blobs().blobs().clone();
        let docs = store.docs().clone();

        seed_manifest_assets(&blobs).await;

        let signing_key = P256KeyPair::generate();
        let did = signing_key.public().to_did();
        let identity = Arc::new(Identity::new(did, signing_key));
        let actor = store.local_actor(identity);

        tx.send(TestWds {
            actor,
            blobs,
            docs,
            store: store.blobs().clone(),
        })
        .await
        .expect("send");
    });

    rx.recv_blocking().expect("wds setup")
}

/// Adds manifest assets sitting in the client's asset directory to the store,
/// so an example resolves them locally instead of needing a provider.
async fn seed_manifest_assets(blobs: &Blobs) {
    for asset in MANIFEST {
        let path = PathBuf::from("../unavi-client/assets").join(asset.rel_path);
        if !path.is_file() {
            continue;
        }
        if let Err(err) = blobs.add_path(&path).await {
            println!("failed to seed {}: {err:?}", asset.rel_path);
        }
    }
}
