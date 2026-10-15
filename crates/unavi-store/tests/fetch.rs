//! Content a joined document references is fetched within the caps, and
//! never past the length its entry claims.

use std::time::Duration;

use bytes::Bytes;
use iroh::{
    Endpoint,
    SecretKey,
    address_lookup::memory::MemoryLookup,
    endpoint::presets::N0DisableRelay,
    protocol::Router,
};
use iroh_docs::{
    Author,
    engine::LiveEvent,
};
use n0_future::StreamExt;
use rstest::rstest;
use tracing_test::traced_test;
use unavi_store::{
    MAX_ENTRY_BYTES,
    Store,
    StoreBuilder,
};

async fn store(lookup: Option<MemoryLookup>) -> (Store, Endpoint, Router) {
    let secret_key = SecretKey::generate();
    let author = Author::from_bytes(&secret_key.to_bytes());

    let mut builder = Endpoint::builder(N0DisableRelay).secret_key(secret_key);
    if let Some(lookup) = lookup {
        builder = builder.address_lookup(lookup);
    }
    let endpoint = builder.bind().await.expect("bind endpoint");

    let store = StoreBuilder::new(endpoint.clone(), author)
        .build()
        .await
        .expect("build store");
    let router = store.accept(Router::builder(endpoint.clone())).spawn();

    (store, endpoint, router)
}

#[rstest]
#[timeout(Duration::from_secs(60))]
#[traced_test]
#[tokio::test(flavor = "multi_thread")]
async fn a_join_fetches_only_what_the_caps_allow() {
    let (host, host_endpoint, _host_router) = store(None).await;

    let doc = host.create().await.expect("create");
    doc.set("honest", "small content").await.expect("honest");

    // Real content, claimed shorter than it is.
    let long = Bytes::from(vec![7; 64 * 1024]);
    let long_hash = host.blobs().add_bytes(long).await.expect("add").hash;
    doc.set_hash("short-claim", long_hash, 1024)
        .await
        .expect("short claim");

    // Small content, claimed past the entry cap.
    let small = host
        .blobs()
        .add_bytes(Bytes::from_static(b"tiny"))
        .await
        .expect("add")
        .hash;
    doc.set_hash("huge-claim", small, MAX_ENTRY_BYTES + 1)
        .await
        .expect("huge claim");

    doc.serve().await.expect("serve");

    let lookup = MemoryLookup::new();
    lookup.add_endpoint_info(host_endpoint.addr());
    let (client, _endpoint, _router) = store(Some(lookup)).await;

    let (joined, mut events) = client
        .join(doc.id(), vec![host_endpoint.addr()])
        .await
        .expect("join");

    while let Some(event) = events.next().await {
        if matches!(event.expect("event"), LiveEvent::PendingContentReady) {
            break;
        }
    }

    assert_eq!(
        joined.get("honest").await.expect("get"),
        Some(Bytes::from_static(b"small content")),
        "content within the caps arrives"
    );
    assert_eq!(
        joined.get("short-claim").await.expect("get"),
        None,
        "content longer than its claim must not complete"
    );
    assert_eq!(
        joined.get("huge-claim").await.expect("get"),
        None,
        "an entry claiming more than the cap is not fetched"
    );
    assert!(
        !client.blobs().has(small).await.expect("has"),
        "nothing was requested for the over-cap entry"
    );
}
