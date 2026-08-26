// Shared by every example binary; each uses one store variant, so the other
// is dead per-binary.
#![expect(dead_code)]

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
use unavi_assets_fetch::MANIFEST;
use unavi_util::{
    async_task::spawn_async_task,
    dirs::data_local_dir,
};
use wds::{
    DataStore,
    actor::Actor,
    identity::{
        WdsIdentity,
        store::KeyStorage,
    },
};

pub struct TestWds {
    pub actor: Actor,
    pub blobs: Blobs,
    pub docs:  Docs,
    pub store: Store,
}

/// An isolated in-memory store for examples that need no manifest assets.
#[must_use]
pub fn create_test_wds() -> TestWds {
    build_wds(None)
}

/// The client's persistent store, which holds the manifest assets it fetched
/// over iroh. Reused so an example resolves them locally with no provider.
#[must_use]
pub fn create_client_wds() -> TestWds {
    build_wds(Some(data_local_dir().join("wds")))
}

fn build_wds(storage: Option<PathBuf>) -> TestWds {
    let (tx, rx) = async_channel::bounded(1);

    spawn_async_task(async move {
        let endpoint = iroh::Endpoint::builder(N0)
            .bind()
            .await
            .expect("iroh endpoint");

        // The persistent store's documents were authored under the client's
        // identity, so an example reading them back has to load the same key.
        let key_storage = if storage.is_some() {
            KeyStorage::Path(data_local_dir().to_path_buf())
        } else {
            KeyStorage::Ephemeral
        };
        let identity = Arc::new(WdsIdentity::load(&key_storage).expect("identity"));

        let builder = DataStore::builder(endpoint.clone(), identity);
        let persistent = storage.is_some();
        let builder = match storage {
            Some(path) => builder.storage_path(path),
            None => builder,
        };
        let (store, f) = builder.build().await.expect("data store");

        let rb = Router::builder(endpoint);
        let rb = f(rb);
        let _router = rb.spawn();

        let blobs = store.blobs().blobs().clone();
        let docs = store.docs().clone();

        if persistent {
            warn_missing_manifest_assets(&blobs).await;
        }

        let actor = store.local_actor();

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
