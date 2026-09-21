//! A blob the local store has never seen is pulled from a sync target, rather
//! than waited on until it happens to arrive.

use std::time::Duration;

use bevy::{
    prelude::*,
    tasks::futures_lite::StreamExt,
};
use bevy_iroh::{
    IrohPlugin,
    blob::request::{
        BlobRequest,
        BlobResponse,
    },
    store::{
        LocalBlobs,
        LocalDownloader,
        SyncTargets,
    },
};
use blake3::Hash;
use bytes::Bytes;
use iroh::{
    Endpoint,
    EndpointAddr,
    SecretKey,
    address_lookup::memory::MemoryLookup,
    endpoint::presets::N0DisableRelay,
    protocol::Router,
};
use iroh_blobs::api::{
    blobs::Blobs,
    downloader::Downloader,
};
use iroh_docs::Author;
use unavi_util::async_task::spawn_async_task;
use wds::builder::StoreBuilder;

const CONTENT: &[u8] = b"content only the provider holds";
/// Bounds the wait for the blob to land in the client's local store; a real
/// network fetch has no fixed duration to guess at.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30);
/// Bounds the catch-up loop that ticks the ECS until the already-landed blob
/// surfaces as a [`BlobResponse`], which takes no real time of its own.
const ATTEMPTS: usize = 200;

struct Fixture {
    blobs:    Blobs,
    hash:     Hash,
    provider: EndpointAddr,
    download: Downloader,
    _routers: Vec<Router>,
}

/// Builds both stores on the shared async runtime, which is the one the fetch
/// task runs on: a quinn endpoint bound to another runtime's IO driver is not
/// usable from this one.
fn fixture() -> Fixture {
    let (tx, rx) = async_channel::bounded(1);

    spawn_async_task(async move {
        let host_key = SecretKey::generate();
        let host_author = Author::from_bytes(&host_key.to_bytes());
        let host_endpoint = Endpoint::builder(N0DisableRelay)
            .secret_key(host_key)
            .bind()
            .await
            .expect("bind host");
        let host = StoreBuilder::new(host_endpoint.clone(), host_author)
            .build()
            .await
            .expect("host store");
        let host_router = (host.router)(Router::builder(host_endpoint.clone())).spawn();

        let hash = host
            .store
            .blobs()
            .add_bytes(Bytes::from_static(CONTENT))
            .await
            .expect("add bytes")
            .hash;

        let lookup = MemoryLookup::new();
        lookup.add_endpoint_info(host_endpoint.addr());

        let client_key = SecretKey::generate();
        let client_author = Author::from_bytes(&client_key.to_bytes());
        let endpoint = Endpoint::builder(N0DisableRelay)
            .address_lookup(lookup)
            .secret_key(client_key)
            .bind()
            .await
            .expect("bind client");
        let store = StoreBuilder::new(endpoint.clone(), client_author)
            .build()
            .await
            .expect("client store");
        let router = (store.router)(Router::builder(endpoint.clone())).spawn();

        let provider = host_endpoint.addr();

        tx.send(Fixture {
            blobs: store.store.blobs().clone(),
            hash: hash.into(),
            provider,
            download: store.store.blob_store().downloader(&endpoint),
            _routers: vec![host_router, router],
        })
        .await
        .expect("send fixture");

        // Both stores stay alive for as long as the test holds the fixture.
        std::future::pending::<()>().await;
    });

    rx.recv_blocking().expect("fixture")
}

/// Blocks until `hash` is fully present in `blobs`, on the store's own
/// completion signal rather than a guessed sleep duration.
///
/// Runs on the runtime the store's endpoint is bound to, same as [`fixture`]:
/// a quinn endpoint's IO driver is not pollable from another runtime.
fn wait_for_download(blobs: &Blobs, hash: Hash) {
    let blobs = blobs.clone();
    let (tx, rx) = async_channel::bounded(1);

    spawn_async_task(async move {
        let arrived = n0_future::time::timeout(DOWNLOAD_TIMEOUT, async {
            let mut stream = blobs.observe(hash).stream().await.expect("observe");
            while let Some(field) = stream.next().await {
                if field.is_complete() {
                    return;
                }
            }
        })
        .await
        .is_ok();
        tx.send(arrived).await.ok();
    });

    assert!(
        rx.recv_blocking().expect("wait task"),
        "the blob never arrived"
    );
}

#[test]
fn a_missing_blob_is_pulled_from_a_sync_target() {
    let fixture = fixture();

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, IrohPlugin));
    app.world_mut().spawn((
        LocalBlobs(fixture.blobs.clone()),
        LocalDownloader(fixture.download.clone()),
        SyncTargets(vec![fixture.provider.clone()]),
    ));

    let entity = app.world_mut().spawn(BlobRequest(fixture.hash)).id();
    app.update();

    wait_for_download(&fixture.blobs, fixture.hash);

    for _ in 0..ATTEMPTS {
        app.update();

        if let Some(response) = app.world().get::<BlobResponse>(entity) {
            let bytes = response.0.as_ref().expect("fetched blob");
            assert_eq!(bytes.as_ref(), CONTENT, "the provider's content arrives");
            return;
        }
    }

    panic!("the blob response never surfaced in the ECS");
}
