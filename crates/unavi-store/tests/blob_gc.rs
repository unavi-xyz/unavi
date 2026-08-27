//! Blob GC deletes anything no tag covers. Document content carries no tag of
//! its own, so without the docs protect callback a GC run reclaims every open
//! document's content.

use std::time::Duration;

use bytes::Bytes;
use n0_future::StreamExt;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::store_with_gc;

mod common;

const GC_INTERVAL: Duration = Duration::from_millis(100);
/// Long enough for several GC passes, so a pass that would delete has run.
const SETTLE: Duration = Duration::from_secs(1);

const KEPT: &[u8] = b"content an open document references";
const DROPPED: &[u8] = b"content nothing references";

#[rstest]
#[timeout(Duration::from_secs(30))]
#[traced_test]
#[tokio::test]
async fn gc_keeps_document_content_and_drops_the_rest() {
    let spawned = store_with_gc(GC_INTERVAL).await;
    let store = &spawned.store;
    let ns = store.create().await.expect("namespace");

    // Added through a batch rather than `add_bytes`, whose default `with_tag`
    // would pin the blob and leave nothing for GC to decide. The batch's temp
    // tag releases when it drops, so nothing roots this afterwards.
    {
        let batch = store.blobs().batch().await.expect("batch");
        let _temp = batch
            .add_bytes(Bytes::from_static(DROPPED))
            .await
            .expect("add dropped");
    }

    ns.set("kept", KEPT).await.expect("write entry");

    n0_future::time::sleep(SETTLE).await;

    assert!(
        store
            .blobs()
            .has(blake3::hash(KEPT))
            .await
            .expect("has kept"),
        "an open document's content must survive gc"
    );
    assert!(
        !store
            .blobs()
            .has(blake3::hash(DROPPED))
            .await
            .expect("has dropped"),
        "content nothing references must be reclaimed"
    );
}

/// A bare `add_bytes(..).await` mints an `auto-<rfc3339>` tag that nothing ever
/// sweeps, so every value ever written stays rooted and every superseded value
/// stays stranded.
#[rstest]
#[timeout(Duration::from_secs(30))]
#[traced_test]
#[tokio::test]
async fn writing_an_entry_leaves_no_named_tag() {
    let spawned = store_with_gc(GC_INTERVAL).await;
    let store = &spawned.store;
    let ns = store.create().await.expect("namespace");

    ns.set("slot", DROPPED).await.expect("first");
    ns.set("slot", KEPT).await.expect("second");

    let mut stream = store.blob_store().tags().list().await.expect("list tags");
    let mut names = Vec::new();
    while let Some(tag) = stream.next().await {
        names.push(tag.expect("tag info").name);
    }
    assert!(
        names.is_empty(),
        "an entry write must mint no tag: {names:?}"
    );

    n0_future::time::sleep(SETTLE).await;

    assert!(
        store
            .blobs()
            .has(blake3::hash(KEPT))
            .await
            .expect("has kept"),
        "the surviving entry's content must outlive its write"
    );
    assert!(
        !store
            .blobs()
            .has(blake3::hash(DROPPED))
            .await
            .expect("has dropped"),
        "an overwritten entry's content must be reclaimed"
    );
}
