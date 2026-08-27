//! A blob the local store has never seen is pulled from a sync target, rather
//! than waited on until it happens to arrive.

use std::time::Duration;

use bevy::prelude::*;
use bevy_iroh::{
    IrohPlugin,
    blob::request::{
        BlobRequest,
        BlobResponse,
    },
    store::{
        LocalBlobStore,
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
use unavi_store::store::Builder as StoreBuilder;
use unavi_util::async_task::spawn_async_task;

const CONTENT: &[u8] = b"content only the provider holds";
const POLL: Duration = Duration::from_millis(50);
const ATTEMPTS: usize = 200;

struct Fixture {
    blobs:    Blobs,
    store:    iroh_blobs::api::Store,
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
            store: store.store.blob_store().clone(),
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

#[test]
fn a_missing_blob_is_pulled_from_a_sync_target() {
    let fixture = fixture();

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, IrohPlugin));
    app.world_mut().spawn((
        LocalBlobs(fixture.blobs.clone()),
        LocalBlobStore(fixture.store.clone()),
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
