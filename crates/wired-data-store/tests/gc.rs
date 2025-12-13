mod common;

use std::str::FromStr;

use common::{
    BLOB_HELLO, DID_ALICE, DID_BOB, DID_CHARLIE, SCHEMA_A, SCHEMA_B, SCHEMA_TEST, create_test_store,
};
use wired_data_store::{DataStore, Genesis};
use xdid::core::did::Did;

#[tokio::test]
async fn test_gc_removes_expired_pins() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");

    store
        .pin_record(&record_id, Some(1))
        .await
        .expect("pin with 1 second expiry");

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let stats = store.garbage_collect().await.expect("run GC");

    assert_eq!(stats.pins_removed, 1, "should remove 1 expired pin");
    assert_eq!(stats.records_removed, 1, "should remove unpinned record");
}

#[tokio::test]
async fn test_gc_keeps_records_with_valid_pins() {
    let (store, _dir) = create_test_store(DID_BOB).await;

    let genesis = Genesis::new(Did::from_str(DID_BOB).expect("parse DID"), SCHEMA_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");

    store
        .pin_record(&record_id, Some(3600))
        .await
        .expect("pin with future expiry");

    let stats = store.garbage_collect().await.expect("run GC");

    assert_eq!(stats.pins_removed, 0, "should not remove valid pins");
    assert_eq!(stats.records_removed, 0, "should not remove pinned records");

    assert!(
        store
            .get_record(&record_id)
            .await
            .expect("query record")
            .is_some(),
        "record should still exist"
    );
}

#[tokio::test]
async fn test_gc_multiple_pins_deletes_when_all_expire() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let store1 = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_ALICE).expect("parse DID"),
    )
    .await
    .expect("create store 1");
    let store2 = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_BOB).expect("parse DID"),
    )
    .await
    .expect("create store 2");

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let record_id = store1.create_record(genesis).await.expect("create record");

    store1
        .pin_record(&record_id, Some(1))
        .await
        .expect("alice pins with 1 second expiry");
    store2
        .pin_record(&record_id, Some(1))
        .await
        .expect("bob pins with 1 second expiry");

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let stats = store1.garbage_collect().await.expect("run GC");

    assert_eq!(stats.pins_removed, 2, "should remove both expired pins");
    assert_eq!(
        stats.records_removed, 1,
        "should remove record when all pins gone"
    );
}

#[tokio::test]
async fn test_gc_null_expiry_never_removed() {
    let (store, _dir) = create_test_store(DID_CHARLIE).await;

    let genesis = Genesis::new(Did::from_str(DID_CHARLIE).expect("parse DID"), SCHEMA_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");

    store
        .pin_record(&record_id, None)
        .await
        .expect("pin without expiry");

    let stats = store.garbage_collect().await.expect("run GC");

    assert_eq!(stats.pins_removed, 0, "should not remove NULL expiry pins");
    assert_eq!(stats.records_removed, 0, "should not remove records");

    assert!(
        store
            .get_record(&record_id)
            .await
            .expect("query record")
            .is_some(),
        "record should still exist"
    );
}

#[tokio::test]
async fn test_gc_removes_orphaned_blobs() {
    let (store, dir) = create_test_store(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");
    let blob_id = store.store_blob(BLOB_HELLO).await.expect("store blob");
    store
        .link_blob_to_record(&record_id, &blob_id)
        .await
        .expect("link blob");

    store
        .pin_record(&record_id, Some(1))
        .await
        .expect("pin record");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let stats = store.garbage_collect().await.expect("run GC");

    assert_eq!(stats.records_removed, 1);
    assert_eq!(stats.blobs_removed, 1);

    let blob_path = dir
        .path()
        .join("blobs")
        .join(&blob_id.as_str()[..2])
        .join(blob_id.as_str());
    assert!(!blob_path.exists(), "blob file should be deleted");
}

#[tokio::test]
async fn test_gc_keeps_blob_with_multiple_record_refs() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let genesis1 = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_A);
    let genesis2 = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_B);
    let record1 = store
        .create_record(genesis1)
        .await
        .expect("create record 1");
    let record2 = store
        .create_record(genesis2)
        .await
        .expect("create record 2");

    let blob_id = store.store_blob(BLOB_HELLO).await.expect("store blob");
    store
        .link_blob_to_record(&record1, &blob_id)
        .await
        .expect("link to record1");
    store
        .link_blob_to_record(&record2, &blob_id)
        .await
        .expect("link to record2");

    store
        .pin_record(&record1, Some(1))
        .await
        .expect("pin record1 with expiry");
    store
        .pin_record(&record2, None)
        .await
        .expect("pin record2 permanently");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let stats = store.garbage_collect().await.expect("run GC");

    assert_eq!(stats.records_removed, 1);
    assert_eq!(
        stats.blobs_removed, 0,
        "blob should remain - still referenced by record2"
    );
    assert!(store.get_blob(&blob_id).expect("get blob").is_some());
}
