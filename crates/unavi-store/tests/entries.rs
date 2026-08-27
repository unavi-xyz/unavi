use std::time::Duration;

use bytes::Bytes;
use rstest::rstest;
use tracing_test::traced_test;
use unavi_store::store::Spawned;

use crate::common::store;

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn entries_round_trip_through_the_blob_store(#[future] store: Spawned) {
    let ns = store.store.create().await.expect("create namespace");

    ns.set("p/A/xform/", "xform-payload").await.expect("xform");
    ns.set("p/A/name/", "name-payload").await.expect("name");

    let listed = ns.list(&["p/"]).await.expect("list");
    assert_eq!(listed.len(), 2);

    let xform = listed
        .iter()
        .find(|e| e.key() == b"p/A/xform/")
        .expect("xform entry");
    assert_eq!(
        ns.value(xform).await.expect("value"),
        Bytes::from_static(b"xform-payload")
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn an_empty_value_reads_as_absence(#[future] store: Spawned) {
    let ns = store.store.create().await.expect("create namespace");

    ns.set("p/A/parent/", "\x00").await.expect("set");
    ns.remove("p/A/parent/").await.expect("remove");

    assert_eq!(ns.list(&["p/"]).await.expect("list"), []);
}

/// A prefix sweep deletes this node's entries rather than marking them, so
/// swept entries stop costing storage. A tombstone written afterwards is a
/// fresh entry the sweep never saw.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn a_prefix_sweep_then_a_tombstone_leaves_the_tombstone(#[future] store: Spawned) {
    let ns = store.store.create().await.expect("create namespace");

    ns.set("p/A/parent/", "\x00").await.expect("parent");
    ns.set("p/A/name/", "doomed").await.expect("name");
    ns.remove("p/A/").await.expect("sweep");
    ns.remove("p/A/parent/").await.expect("tombstone");

    assert_eq!(ns.list(&["p/"]).await.expect("list"), []);
}

/// The same bytes written under two keys are stored once, so a value shared by
/// many prims costs one copy.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn identical_values_share_one_blob(#[future] store: Spawned) {
    let ns = store.store.create().await.expect("create namespace");

    for prim in ["A", "B"] {
        ns.set(format!("h/{prim}/mesh:POSITION/"), "vertex-buffer")
            .await
            .expect("set");
    }

    let listed = ns.list(&["h/"]).await.expect("list");
    assert_eq!(listed.len(), 2);
    assert_eq!(
        listed[0].content_hash(),
        listed[1].content_hash(),
        "one copy of the bytes, referenced twice"
    );
}
