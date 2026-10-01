//! Nothing else in the process closes a replica, so a
//! [`unavi_store::Document`] that never released its handle would
//! make eviction a no-op rather than merely slow.

use std::time::Duration;

use rstest::rstest;
use tracing_test::traced_test;
use unavi_store::Store;

use crate::common::store;

mod common;

#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn releasing_documents_frees_the_replica(#[future] store: Store) {
    let ns = store.create().await.expect("create").id();

    for _ in 0..8 {
        store
            .held(ns)
            .await
            .expect("held")
            .expect("a created document is held");
    }

    store
        .remove(ns)
        .await
        .expect("every released document must close the handle it took");

    assert!(
        !store
            .list()
            .await
            .expect("list")
            .iter()
            .any(|(held, _)| *held == ns),
        "a dropped replica must be gone from what this store holds"
    );
}

/// Safety rule: a sweep must not delete a document out from under whatever is
/// still using it.
#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn a_held_document_refuses_to_be_removed(#[future] store: Store) {
    let held = store.create().await.expect("create");

    store
        .remove(held.id())
        .await
        .expect_err("a replica still held must not be deleted");

    held.set("key", "value")
        .await
        .expect("the document that survived the remove is still usable");
}
