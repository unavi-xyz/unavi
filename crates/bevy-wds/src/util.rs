use std::sync::Arc;

use iroh::{endpoint::presets::N0, protocol::Router};
use wds::{Blobs, DataStore, Identity, actor::Actor};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

/// Spawns a WDS and actor on a new thread.
/// Useful for testing or examples.
#[must_use]
pub fn create_test_wds() -> (Actor, Blobs) {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    unavi_wasm_compat::spawn_thread(async move {
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

        let signing_key = P256KeyPair::generate();
        let did = signing_key.public().to_did();
        let identity = Arc::new(Identity::new(did, signing_key));
        let actor = store.local_actor(identity);

        tx.send((actor, blobs)).expect("send");

        // Keep runtime (and data store) alive for the process lifetime.
        std::future::pending::<()>().await;
    });

    rx.recv().expect("wds setup")
}
