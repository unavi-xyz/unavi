use std::time::Duration;

use bytes::Bytes;
use iroh_docs::api::Doc;
use rstest::rstest;
use tracing_test::traced_test;
use wds::entries::{
    self,
    Write,
};

use crate::common::{
    DataStoreCtx,
    ctx,
};

mod common;

async fn doc(ctx: &DataStoreCtx) -> Doc {
    let ns = entries::create(ctx.store.docs())
        .await
        .expect("create namespace");
    ctx.store
        .docs()
        .api()
        .open(ns)
        .await
        .expect("open")
        .expect("doc present")
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_entries_round_trip(#[future] ctx: DataStoreCtx) {
    let doc = doc(&ctx).await;
    let blobs = ctx.store.blobs().blobs();
    let author = entries::author(ctx.store.docs()).await.expect("author");

    entries::apply(
        &doc,
        blobs,
        author,
        [
            Write::Bytes {
                key:   "p/A/xform/".to_owned(),
                value: Bytes::from_static(b"xform-payload"),
            },
            Write::Bytes {
                key:   "p/A/name/".to_owned(),
                value: Bytes::from_static(b"name-payload"),
            },
        ],
    )
    .await
    .expect("apply");

    let listed = entries::list(&doc, &["p/"]).await.expect("list");
    assert_eq!(listed.len(), 2);

    let xform = listed
        .iter()
        .find(|e| e.key == "p/A/xform/")
        .expect("xform entry");
    assert_eq!(
        entries::value(blobs, xform).await.expect("value"),
        Bytes::from_static(b"xform-payload")
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_empty_value_reads_as_absence(#[future] ctx: DataStoreCtx) {
    let doc = doc(&ctx).await;
    let blobs = ctx.store.blobs().blobs();
    let author = entries::author(ctx.store.docs()).await.expect("author");

    entries::apply(
        &doc,
        blobs,
        author,
        [
            Write::Bytes {
                key:   "p/A/parent/".to_owned(),
                value: Bytes::from_static(b"\x00"),
            },
            Write::Remove {
                key: "p/A/parent/".to_owned(),
            },
        ],
    )
    .await
    .expect("apply");

    assert!(entries::list(&doc, &["p/"]).await.expect("list").is_empty());
}

/// A prefix wipe removes rather than marks, so the entries it sweeps stop
/// costing storage — but it would also eat a tombstone written beneath it,
/// which is why deletion tombstones the parent last.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_sweep_then_tombstone_leaves_the_tombstone(#[future] ctx: DataStoreCtx) {
    let doc = doc(&ctx).await;
    let blobs = ctx.store.blobs().blobs();
    let author = entries::author(ctx.store.docs()).await.expect("author");

    entries::apply(
        &doc,
        blobs,
        author,
        [
            Write::Bytes {
                key:   "p/A/parent/".to_owned(),
                value: Bytes::from_static(b"\x00"),
            },
            Write::Bytes {
                key:   "p/A/name/".to_owned(),
                value: Bytes::from_static(b"doomed"),
            },
            Write::Remove {
                key: "p/A/".to_owned(),
            },
            Write::Remove {
                key: "p/A/parent/".to_owned(),
            },
        ],
    )
    .await
    .expect("apply");

    assert!(entries::list(&doc, &["p/"]).await.expect("list").is_empty());
}

/// Hash entries reference a blob the caller already holds, so the same bytes
/// referenced by many prims deduplicate in the blob store.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_hash_entries_share_one_blob(#[future] ctx: DataStoreCtx) {
    let doc = doc(&ctx).await;
    let blobs = ctx.store.blobs().blobs();
    let author = entries::author(ctx.store.docs()).await.expect("author");

    let payload = Bytes::from_static(b"vertex-buffer");
    let info = blobs.add_bytes(payload.clone()).await.expect("add bytes");

    entries::apply(
        &doc,
        blobs,
        author,
        ["A", "B"].map(|prim| Write::Hash {
            key:  format!("h/{prim}/mesh:POSITION/"),
            hash: info.hash,
            size: payload.len() as u64,
        }),
    )
    .await
    .expect("apply");

    let listed = entries::list(&doc, &["h/"]).await.expect("list");
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().all(|entry| entry.hash == info.hash));
    assert!(entries::list(&doc, &["p/"]).await.expect("list").is_empty());
}
