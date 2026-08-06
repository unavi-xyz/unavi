use std::sync::Arc;

use iroh::{
    endpoint::presets::N0,
    protocol::Router,
};
use iroh_blobs::api::blobs::Blobs;
use iroh_docs::protocol::Docs;
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

#[must_use]
pub fn create_test_wds() -> (Actor, Docs, Blobs) {
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

        let signing_key = P256KeyPair::generate();
        let did = signing_key.public().to_did();
        let identity = Arc::new(Identity::new(did, signing_key));
        let actor = store.local_actor(identity);

        tx.send((actor, docs, blobs)).await.expect("send");
    });

    rx.recv_blocking().expect("wds setup")
}
