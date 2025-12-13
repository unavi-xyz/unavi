mod common;

use std::str::FromStr;

use common::{DID_ALICE, create_test_store, create_test_view};
use wired_data_store::Genesis;
use xdid::core::did::Did;

#[tokio::test]
async fn test_empty_blob() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let empty: &[u8] = b"";
    let blob_id = view.store_blob(empty).await.expect("store empty blob");

    let retrieved = view.get_blob(&blob_id).expect("get blob").expect("exists");
    assert!(retrieved.is_empty());
}

#[tokio::test]
async fn test_empty_schema() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), "");
    let id = view.create_record(genesis).await.expect("create record");

    let record = view
        .get_record(&id)
        .await
        .expect("get record")
        .expect("exists");
    assert!(record.genesis.schema.is_empty());
}

#[tokio::test]
async fn test_unicode_schema() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let unicode_schema = "スキーマ-схема-架構";
    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), unicode_schema);
    let id = view.create_record(genesis).await.expect("create record");

    let record = view
        .get_record(&id)
        .await
        .expect("get record")
        .expect("exists");
    assert_eq!(record.genesis.schema.as_str(), unicode_schema);
}

#[tokio::test]
async fn test_very_long_schema() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let long_schema = "x".repeat(10_000);
    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), &long_schema);
    let id = view.create_record(genesis).await.expect("create record");

    let record = view
        .get_record(&id)
        .await
        .expect("get record")
        .expect("exists");
    assert_eq!(record.genesis.schema.as_str(), long_schema);
}

#[tokio::test]
async fn test_binary_blob_content() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    // All byte values including null bytes.
    let binary_data: Vec<u8> = (0u8..=255).collect();
    let blob_id = view.store_blob(&binary_data).await.expect("store blob");

    let retrieved = view.get_blob(&blob_id).expect("get blob").expect("exists");
    assert_eq!(retrieved, binary_data);
}

#[tokio::test]
async fn test_duplicate_blob_storage() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let data = b"duplicate content";
    let id1 = view.store_blob(data).await.expect("store first");
    let id2 = view.store_blob(data).await.expect("store second");

    // Should return same CID (content-addressed).
    assert_eq!(id1, id2);

    // Should still be retrievable.
    let retrieved = view.get_blob(&id1).expect("get blob").expect("exists");
    assert_eq!(&retrieved, data);
}

#[tokio::test]
async fn test_pin_then_unpin_then_gc() {
    let (store, _dir) = create_test_store().await;
    let view = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), "test");
    let id = view.create_record(genesis).await.expect("create");

    view.pin_record(&id, None).await.expect("pin");
    view.unpin_record(&id).await.expect("unpin");

    // Record is unpinned, but since it was explicitly created, GC behavior
    // depends on whether there are expired pins. With no pin expiry,
    // remove_expired_pins won't flag it for deletion.
    let stats = store.garbage_collect().await.expect("gc");

    // No expired pins to trigger deletion.
    assert_eq!(stats.pins_removed, 0);
}
