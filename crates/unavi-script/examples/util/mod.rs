// Shared by every example binary; each uses one store variant, so the other
// is dead per-binary.
#![expect(dead_code)]

use std::sync::Arc;

use iroh::{
    endpoint::presets::N0,
    protocol::Router,
};
use iroh_blobs::api::{
    Store as BlobStore,
    blobs::Blobs,
};
use iroh_docs::protocol::Docs;
use unavi_assets_fetch::MANIFEST;
use unavi_identity::identity::{
    Identity,
    NodeIdentity,
};
use unavi_store::{
    builder::{
        Builder as StoreBuilder,
        Store,
    },
    local::Storage,
};
use unavi_util::{
    async_task::spawn_async_task,
    dirs::data_local_dir,
};

pub struct TestStore {
    pub blobs:    Blobs,
    pub docs:     Docs,
    pub identity: Arc<Identity>,
    pub store:    BlobStore,
}

/// An isolated in-memory store for examples that need no manifest assets.
#[must_use]
pub fn create_test_store() -> TestStore {
    build(false)
}

/// The client's persistent store, which holds the manifest assets it fetched
/// over iroh. Reused so an example resolves them locally with no provider.
#[must_use]
pub fn create_client_store() -> TestStore {
    build(true)
}

fn build(persistent: bool) -> TestStore {
    let (tx, rx) = async_channel::bounded(1);

    spawn_async_task(async move {
        // The persistent store's documents were authored under the client's
        // identity, so an example reading them back has to load the same key.
        let storage = if persistent {
            Storage::Path(data_local_dir().to_path_buf())
        } else {
            Storage::Ephemeral
        };
        let node = NodeIdentity::load(&storage).expect("identity");

        let endpoint = iroh::Endpoint::builder(N0)
            .secret_key(node.endpoint().clone())
            .bind()
            .await
            .expect("iroh endpoint");

        let builder = StoreBuilder::new(endpoint.clone(), node.author()).storage(storage);
        let Store {
            blobs: blob_store,
            docs,
            router,
            guard: _guard,
            ..
        } = builder.build().await.expect("data store");

        let rb = Router::builder(endpoint);
        let rb = router(rb);
        let _router = rb.spawn();

        let blobs = blob_store.blobs().clone();

        if persistent {
            warn_missing_manifest_assets(&blobs).await;
        }

        tx.send(TestStore {
            blobs,
            docs,
            identity: Arc::clone(node.user()),
            store: blob_store,
        })
        .await
        .expect("send");

        // `_guard` stays in scope: dropping it shuts the blob store down, so
        // the example holds it for as long as it runs.
        std::future::pending::<()>().await;
    });

    rx.recv_blocking().expect("store setup")
}

/// The client fetches manifest assets over iroh into its own store; one it
/// has not pulled yet leaves the avatar — and with it the agent's camera
/// proxy — waiting on a provider the harness does not have.
async fn warn_missing_manifest_assets(blobs: &Blobs) {
    for asset in MANIFEST {
        let hash = blake3::Hash::from_hex(asset.hash).expect("manifest hash");
        if blobs.has(hash).await.unwrap_or(false) {
            continue;
        }
        println!(
            "missing manifest asset, run the client once to fetch it: {}",
            asset.rel_path
        );
    }
}
