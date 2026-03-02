use std::sync::Arc;

use wds::{Blobs, DataStore, Identity, actor::Actor};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

/// Spawns a WDS and actor on a new thread.
/// Useful for testing or examples.
///
/// # Panics
///
/// Panics if the tokio runtime could not be initialized.
#[must_use]
pub fn create_test_wds() -> (Actor, Blobs) {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");

        rt.block_on(async {
            let endpoint = iroh::Endpoint::builder()
                .bind()
                .await
                .expect("iroh endpoint");

            let store = DataStore::builder(endpoint)
                .build()
                .await
                .expect("data store");

            let blobs = store.blobs().blobs().clone();

            let signing_key = P256KeyPair::generate();
            let did = signing_key.public().to_did();
            let identity = Arc::new(Identity::new(did, signing_key));
            let actor = store.local_actor(identity);

            tx.send((actor, blobs)).expect("send");

            // Keep runtime (and data store) alive for the process lifetime.
            std::future::pending::<()>().await;
        });
    });

    rx.recv().expect("wds setup")
}
