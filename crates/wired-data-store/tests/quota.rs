mod common;

use std::str::FromStr;

use common::{BLOB_HELLO, DID_ALICE, DID_BOB, SCHEMA_TEST, create_test_store};
use wired_data_store::{DataStore, Genesis, DEFAULT_QUOTA_BYTES};
use xdid::core::did::Did;

#[tokio::test]
async fn test_default_quota_is_5gb() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let quota = store.get_storage_quota().await.expect("get quota");

    assert_eq!(quota.quota_bytes, DEFAULT_QUOTA_BYTES);
    assert_eq!(quota.quota_bytes, 5 * 1024 * 1024 * 1024);
    assert_eq!(quota.bytes_used, 0);
}

#[tokio::test]
async fn test_record_creation_consumes_quota() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let initial = store.get_storage_quota().await.expect("get quota");
    assert_eq!(initial.bytes_used, 0);

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    store.create_record(genesis).await.expect("create record");

    let after = store.get_storage_quota().await.expect("get quota");
    assert!(after.bytes_used > 0, "quota should increase after record creation");
}

#[tokio::test]
async fn test_blob_upload_consumes_quota() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let initial = store.get_storage_quota().await.expect("get quota");
    assert_eq!(initial.bytes_used, 0);

    store.store_blob(BLOB_HELLO).await.expect("store blob");

    let after = store.get_storage_quota().await.expect("get quota");
    assert_eq!(
        after.bytes_used,
        i64::try_from(BLOB_HELLO.len()).expect("blob size fits in i64"),
        "quota should increase by blob size"
    );
}

#[tokio::test]
async fn test_same_blob_uploaded_twice_charges_once() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    store.store_blob(BLOB_HELLO).await.expect("store blob first time");
    let after_first = store.get_storage_quota().await.expect("get quota");

    store.store_blob(BLOB_HELLO).await.expect("store blob second time");
    let after_second = store.get_storage_quota().await.expect("get quota");

    assert_eq!(
        after_first.bytes_used, after_second.bytes_used,
        "uploading same blob twice should not increase quota"
    );
}

#[tokio::test]
async fn test_record_deletion_releases_quota() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let id = store.create_record(genesis).await.expect("create record");

    let after_create = store.get_storage_quota().await.expect("get quota");
    assert!(after_create.bytes_used > 0);

    store.delete_record(&id).await.expect("delete record");

    let after_delete = store.get_storage_quota().await.expect("get quota");
    assert_eq!(after_delete.bytes_used, 0, "quota should be released after deletion");
}

#[tokio::test]
async fn test_quota_exceeded_returns_error() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    // Set a tiny quota.
    store.set_storage_quota(10).await.expect("set quota");

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let result = store.create_record(genesis).await;

    assert!(result.is_err(), "should fail when quota exceeded");
    let err = result.expect_err("should fail when quota exceeded");
    assert!(
        err.to_string().contains("quota exceeded"),
        "error should mention quota exceeded: {err}"
    );
}

#[tokio::test]
async fn test_custom_quota_setting() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let custom_quota = 1024 * 1024; // 1 MB
    store.set_storage_quota(custom_quota).await.expect("set quota");

    let quota = store.get_storage_quota().await.expect("get quota");
    assert_eq!(quota.quota_bytes, custom_quota);
}

#[tokio::test]
async fn test_gc_releases_blob_quota() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    // Create a record and link a blob to it.
    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");
    let blob_id = store.store_blob(BLOB_HELLO).await.expect("store blob");
    store
        .link_blob_to_record(&record_id, &blob_id)
        .await
        .expect("link blob");

    // Pin the record, then delete it (which orphans the blob).
    store.pin_record(&record_id, Some(1)).await.expect("pin record");

    let before_gc = store.get_storage_quota().await.expect("get quota");
    assert!(before_gc.bytes_used > 0);

    // Wait for pin to expire and run GC.
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    store.garbage_collect().await.expect("gc");

    let after_gc = store.get_storage_quota().await.expect("get quota");
    assert_eq!(after_gc.bytes_used, 0, "quota should be released after GC");
}

#[tokio::test]
async fn test_quota_isolation_between_users() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let alice = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_ALICE).expect("parse DID"),
    )
    .await
    .expect("create alice store");

    let bob = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_BOB).expect("parse DID"),
    )
    .await
    .expect("create bob store");

    // Alice uploads a blob.
    alice.store_blob(BLOB_HELLO).await.expect("alice stores blob");

    let alice_quota = alice.get_storage_quota().await.expect("alice quota");
    let bob_quota = bob.get_storage_quota().await.expect("bob quota");

    assert!(alice_quota.bytes_used > 0, "alice should have used quota");
    assert_eq!(bob_quota.bytes_used, 0, "bob should have zero quota used");
}

#[tokio::test]
async fn test_bytes_remaining() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    store.set_storage_quota(100).await.expect("set quota");
    store.store_blob(b"test").await.expect("store blob");

    let quota = store.get_storage_quota().await.expect("get quota");
    assert_eq!(quota.bytes_remaining(), 100 - 4);
}
