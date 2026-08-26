//! Blob GC deletes anything no tag covers. Document content carries no tag of
//! its own, so without the docs protect callback a GC run reclaims every open
//! document's content.

use std::time::Duration;

use bytes::Bytes;
use n0_future::StreamExt;
use rstest::rstest;
use tracing_test::traced_test;
use unavi_store::entries::Write;

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
    let store = store_with_gc(GC_INTERVAL).await;
    let blobs = store.blobs.blobs();

    let ns = store.docs.api().create().await.expect("namespace").id();
    let doc = unavi_store::namespace::ensure_open(&store.docs, ns)
        .await
        .expect("doc");
    let author = store.docs.api().author_default().await.expect("author");

    // Added through a batch rather than `add_bytes`, whose default `with_tag`
    // would pin both blobs and leave nothing for gc to decide.
    let (kept, dropped) = {
        let batch = blobs.batch().await.expect("batch");
        let kept = batch
            .add_bytes(Bytes::from_static(KEPT))
            .await
            .expect("add kept")
            .hash();
        let dropped = batch
            .add_bytes(Bytes::from_static(DROPPED))
            .await
            .expect("add dropped")
            .hash();

        // Written while the batch still holds the content, so no gc pass can
        // land between the add and the reference that protects it.
        unavi_store::entries::apply(
            &doc,
            blobs,
            author,
            [Write::Hash {
                key:  "kept".to_string(),
                hash: kept,
                size: KEPT.len() as u64,
            }],
        )
        .await
        .expect("write entry");

        (kept, dropped)
    };

    n0_future::time::sleep(SETTLE).await;

    assert!(
        blobs.has(kept).await.expect("has kept"),
        "an open document's content must survive gc"
    );
    assert!(
        !blobs.has(dropped).await.expect("has dropped"),
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
    let store = store_with_gc(GC_INTERVAL).await;
    let blobs = store.blobs.blobs();

    let ns = store.docs.api().create().await.expect("namespace").id();
    let doc = unavi_store::namespace::ensure_open(&store.docs, ns)
        .await
        .expect("doc");
    let author = store.docs.api().author_default().await.expect("author");

    unavi_store::entries::apply(
        &doc,
        blobs,
        author,
        [
            Write::Bytes {
                key:   "slot".to_string(),
                value: Bytes::from_static(DROPPED),
            },
            Write::Bytes {
                key:   "slot".to_string(),
                value: Bytes::from_static(KEPT),
            },
        ],
    )
    .await
    .expect("write entries");

    let mut stream = store.blobs.tags().list().await.expect("list tags");
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
        blobs.has(blake3::hash(KEPT)).await.expect("has kept"),
        "the surviving entry's content must outlive its batch"
    );
    assert!(
        !blobs.has(blake3::hash(DROPPED)).await.expect("has dropped"),
        "an overwritten entry's content must be reclaimed"
    );
}
