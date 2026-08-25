//! A blob the local store has never seen is pulled from a sync target, rather
//! than waited on until it happens to arrive.

use std::{
    sync::Arc,
    time::Duration,
};

use bevy::prelude::*;
use bevy_wds::{
    LocalBlobs,
    LocalDownloader,
    SyncTargets,
    WdsPlugin,
    blob::request::{
        BlobRequest,
        BlobResponse,
    },
};
use blake3::Hash;
use bytes::Bytes;
use iroh::{
    Endpoint,
    address_lookup::memory::MemoryLookup,
    endpoint::presets::N0DisableRelay,
    protocol::Router,
};
use iroh_blobs::api::{
    blobs::Blobs,
    downloader::Downloader,
};
use unavi_util::async_task::spawn_async_task;
use wds::{
    DataStore,
    actor::Actor,
    identity::{
        RootIdentity,
        store::KeyStorage,
    },
};

const CONTENT: &[u8] = b"content only the provider holds";
const POLL: Duration = Duration::from_millis(50);
const ATTEMPTS: usize = 200;

struct Fixture {
    blobs:    Blobs,
    hash:     Hash,
    provider: Actor,
    download: Downloader,
    _routers: Vec<Router>,
}

fn ephemeral_identity() -> Arc<RootIdentity> {
    Arc::new(RootIdentity::load(&KeyStorage::Ephemeral).expect("generate identity"))
}

/// Builds both stores on the shared async runtime, which is the one the fetch
/// task runs on: a quinn endpoint bound to another runtime's IO driver is not
/// usable from this one.
fn fixture() -> Fixture {
    let (tx, rx) = async_channel::bounded(1);

    spawn_async_task(async move {
        let host_endpoint = Endpoint::builder(N0DisableRelay)
            .bind()
            .await
            .expect("bind host");
        let (host, host_router_fn) =
            DataStore::builder(host_endpoint.clone(), ephemeral_identity())
                .build()
                .await
                .expect("host store");
        let host_router = host_router_fn(Router::builder(host_endpoint.clone())).spawn();

        let hash = host
            .blobs()
            .blobs()
            .add_bytes(Bytes::from_static(CONTENT))
            .await
            .expect("add bytes")
            .hash;

        let lookup = MemoryLookup::new();
        lookup.add_endpoint_info(host_endpoint.addr());

        let endpoint = Endpoint::builder(N0DisableRelay)
            .address_lookup(lookup)
            .bind()
            .await
            .expect("bind client");
        let (store, router_fn) = DataStore::builder(endpoint.clone(), ephemeral_identity())
            .build()
            .await
            .expect("client store");
        let router = router_fn(Router::builder(endpoint.clone())).spawn();

        let provider = store.remote_actor(host_endpoint.addr());

        tx.send(Fixture {
            blobs: store.blobs().blobs().clone(),
            hash: hash.into(),
            provider,
            download: store.blobs().downloader(&endpoint),
            _routers: vec![host_router, router],
        })
        .await
        .expect("send fixture");

        // Both stores stay alive for as long as the test holds the fixture.
        std::future::pending::<()>().await;
    });

    rx.recv_blocking().expect("fixture")
}

#[test]
fn a_missing_blob_is_pulled_from_a_sync_target() {
    let fixture = fixture();

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, WdsPlugin));
    app.world_mut().spawn((
        LocalBlobs(fixture.blobs.clone()),
        LocalDownloader(fixture.download.clone()),
        SyncTargets(vec![fixture.provider.clone()]),
    ));

    let entity = app.world_mut().spawn(BlobRequest(fixture.hash)).id();

    for _ in 0..ATTEMPTS {
        app.update();

        if let Some(response) = app.world().get::<BlobResponse>(entity) {
            let bytes = response.0.as_ref().expect("fetched blob");
            assert_eq!(bytes.as_ref(), CONTENT, "the provider's content arrives");
            return;
        }

        std::thread::sleep(POLL);
    }

    panic!("the blob never arrived");
}
