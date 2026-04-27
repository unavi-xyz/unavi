use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use directories::ProjectDirs;
use iroh::{endpoint::presets::N0, protocol::Router};
use unavi_util::async_task::spawn_async_task;
use wds::{Blobs, DataStore, Identity, actor::Actor};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub fn assets_dir() -> PathBuf {
    DIRS.data_local_dir().join("assets")
}

pub fn copy_assets_to_project_dir(paths: &[&str]) {
    let assets = assets_dir();
    for path in paths {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../unavi-client/assets")
            .join(path);
        let dst = assets.join(path);
        if let Some(parent) = dst.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::copy(&src, &dst) {
            eprintln!("failed to copy {path}: {e}");
        }
    }
}

#[must_use]
pub fn create_test_wds() -> (Actor, Blobs) {
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

        let signing_key = P256KeyPair::generate();
        let did = signing_key.public().to_did();
        let identity = Arc::new(Identity::new(did, signing_key));
        let actor = store.local_actor(identity);

        tx.send((actor, blobs)).await.expect("send");
    });

    rx.recv_blocking().expect("wds setup")
}
