use std::time::Duration;

use rstest::rstest;
use tempfile::tempdir;
use tracing_test::traced_test;
use unavi_store::store::Spawned;

use crate::common::{
    store,
    store_at,
};

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[traced_test]
#[tokio::test]
async fn a_recorded_namespace_reopens() {
    let dir = tempdir().expect("temp dir");
    let store = store_at(dir.path()).await.store;

    let first = store.open_or_mint("view").await.expect("mint").id();
    let second = store.open_or_mint("view").await.expect("reopen").id();

    assert_eq!(
        first, second,
        "a recorded id must name the same document on every open"
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[traced_test]
#[tokio::test]
async fn separate_keys_hold_separate_namespaces() {
    let dir = tempdir().expect("temp dir");
    let store = store_at(dir.path()).await.store;

    let catalog = store
        .open_or_mint("registry/catalog")
        .await
        .expect("mint catalog");
    let recent = store
        .open_or_mint("registry/views/recent")
        .await
        .expect("mint view");

    assert_ne!(catalog.id(), recent.id());
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[traced_test]
#[tokio::test]
async fn an_unheld_namespace_is_reminted() {
    let dir = tempdir().expect("temp dir");
    let store = store_at(dir.path()).await.store;

    let stale = store.open_or_mint("view").await.expect("mint").id();
    store.docs().api().drop_doc(stale).await.expect("drop");

    let minted = store.open_or_mint("view").await.expect("remint");

    assert_ne!(
        minted.id(),
        stale,
        "an id whose capability is gone names an unrecoverable document"
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn in_memory_storage_reopens_within_a_process(#[future] store: Spawned) {
    let first = store.store.open_or_mint("view").await.expect("mint");
    let second = store.store.open_or_mint("view").await.expect("mint");

    assert_eq!(
        first.id(),
        second.id(),
        "a recorded id must name the same document within one process"
    );
}

/// `open` imports a read capability rather than opening, so it has to merge
/// into a write capability this node already holds. A downgrade would leave
/// every document silently read-only on the node that authored it.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn open_keeps_a_held_write_capability(#[future] store: Spawned) {
    let created = store.store.create().await.expect("create");

    let reopened = store.store.open(created.id()).await.expect("open");

    reopened
        .set("written-after-import", "payload")
        .await
        .expect("a namespace this node authored stays writable after open");

    assert_eq!(
        reopened
            .list(&["written-after-import"])
            .await
            .expect("list")
            .len(),
        1
    );
}
