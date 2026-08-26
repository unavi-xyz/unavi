use std::time::Duration;

use rstest::rstest;
use tempfile::tempdir;
use tracing_test::traced_test;
use unavi_store::{
    builder::Store,
    local::Storage,
    namespace,
};

use crate::common::store;

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn a_recorded_namespace_reopens(#[future] store: Store) {
    let dir = tempdir().expect("temp dir");
    let storage = Storage::Path(dir.path().to_path_buf());

    let first = namespace::open_or_mint(&store.docs, &storage, "root-doc")
        .await
        .expect("mint");
    let second = namespace::open_or_mint(&store.docs, &storage, "root-doc")
        .await
        .expect("reopen");

    assert_eq!(
        first, second,
        "a recorded id must name the same document on every open"
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn separate_keys_hold_separate_namespaces(#[future] store: Store) {
    let dir = tempdir().expect("temp dir");
    let storage = Storage::Path(dir.path().to_path_buf());

    let catalog = namespace::open_or_mint(&store.docs, &storage, "registry/catalog")
        .await
        .expect("mint catalog");
    let recent = namespace::open_or_mint(&store.docs, &storage, "registry/views/recent")
        .await
        .expect("mint view");

    assert_ne!(catalog, recent);
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn an_unheld_namespace_is_reminted(#[future] store: Store) {
    let dir = tempdir().expect("temp dir");
    let storage = Storage::Path(dir.path().to_path_buf());

    let stale = store.docs.api().create().await.expect("create").id();
    storage
        .write("root-doc", &stale.to_string())
        .expect("record");
    store.docs.api().drop_doc(stale).await.expect("drop");

    let minted = namespace::open_or_mint(&store.docs, &storage, "root-doc")
        .await
        .expect("remint");

    assert_ne!(
        minted, stale,
        "an id whose capability is gone names an unrecoverable document"
    );
    assert_eq!(
        storage.read("root-doc").as_deref(),
        Some(minted.to_string().as_str()),
        "the replacement must be recorded in place of the lost id"
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn ephemeral_storage_mints_every_run(#[future] store: Store) {
    let first = namespace::open_or_mint(&store.docs, &Storage::Ephemeral, "root-doc")
        .await
        .expect("mint");
    let second = namespace::open_or_mint(&store.docs, &Storage::Ephemeral, "root-doc")
        .await
        .expect("mint");

    assert_ne!(first, second);
}
