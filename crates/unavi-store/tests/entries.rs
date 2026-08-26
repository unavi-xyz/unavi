use std::time::Duration;

use bytes::Bytes;
use iroh_docs::api::Doc;
use rstest::rstest;
use tracing_test::traced_test;
use unavi_store::{
    builder::Store,
    entries::{
        self,
        Write,
    },
};

use crate::common::store;

mod common;

async fn doc(store: &Store) -> Doc {
    let ns = store
        .docs
        .api()
        .create()
        .await
        .expect("create namespace")
        .id();
    store
        .docs
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
async fn test_entries_round_trip(#[future] store: Store) {
    let doc = doc(&store).await;
    let blobs = store.blobs.blobs();
    let author = store.docs.api().author_default().await.expect("author");

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
async fn test_empty_value_reads_as_absence(#[future] store: Store) {
    let doc = doc(&store).await;
    let blobs = store.blobs.blobs();
    let author = store.docs.api().author_default().await.expect("author");

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

    assert_eq!(
        entries::list(&doc, &["p/"]).await.expect("list"),
        [] as [entries::DocEntry; 0]
    );
}

/// A prefix wipe removes rather than marks, so swept entries stop costing
/// storage; it would also eat a tombstone written beneath it.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_sweep_then_tombstone_leaves_the_tombstone(#[future] store: Store) {
    let doc = doc(&store).await;
    let blobs = store.blobs.blobs();
    let author = store.docs.api().author_default().await.expect("author");

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

    assert_eq!(
        entries::list(&doc, &["p/"]).await.expect("list"),
        [] as [entries::DocEntry; 0]
    );
}

/// Hash entries reference a blob the caller already holds, so the same bytes
/// referenced by many prims deduplicate in the blob store.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_hash_entries_share_one_blob(#[future] store: Store) {
    let doc = doc(&store).await;
    let blobs = store.blobs.blobs();
    let author = store.docs.api().author_default().await.expect("author");

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
    assert_eq!(
        entries::list(&doc, &["p/"]).await.expect("list"),
        [] as [entries::DocEntry; 0]
    );
}
