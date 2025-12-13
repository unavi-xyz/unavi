mod common;

use std::str::FromStr;

use common::{DID_ALICE, DID_BOB, SCHEMA_TEST, create_test_store, create_test_view};
use wired_data_store::{Genesis, MAX_BLOB_SIZE};
use xdid::core::did::Did;

#[tokio::test]
async fn test_blob_size_limit_enforced() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let oversized = vec![0u8; MAX_BLOB_SIZE + 1];
    let result = view.store_blob(&oversized).await;

    assert!(result.is_err());
    let err = result.expect_err("oversized blob should fail").to_string();
    assert!(err.contains("exceeds maximum"), "error: {err}");
}

#[tokio::test]
async fn test_blob_at_exact_limit_allowed() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let max_size = vec![0u8; MAX_BLOB_SIZE];
    let result = view.store_blob(&max_size).await;

    assert!(result.is_ok(), "blob at exact limit should be allowed");
}

#[tokio::test]
async fn test_cross_user_record_access_denied() {
    let (store, _dir) = create_test_store().await;

    let view_alice = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));
    let view_bob = store.view_for_user(Did::from_str(DID_BOB).expect("parse DID"));

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let record_id = view_alice
        .create_record(genesis)
        .await
        .expect("alice creates record");

    // Bob should not see Alice's record.
    let result = view_bob.get_record(&record_id).await.expect("query");
    assert!(result.is_none(), "bob should not see alice's record");

    // Bob should not be able to delete Alice's record.
    view_bob
        .delete_record(&record_id)
        .await
        .expect("delete attempt");

    // Alice's record should still exist.
    let record = view_alice
        .get_record(&record_id)
        .await
        .expect("query")
        .expect("record exists");
    assert_eq!(record.id, record_id);
}

#[tokio::test]
async fn test_cross_user_pin_isolation() {
    let (store, _dir) = create_test_store().await;

    let view_alice = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));
    let view_bob = store.view_for_user(Did::from_str(DID_BOB).expect("parse DID"));

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let record_id = view_alice
        .create_record(genesis)
        .await
        .expect("alice creates record");

    // Alice pins her record.
    view_alice
        .pin_record(&record_id, None)
        .await
        .expect("alice pins");

    // Bob unpins (his own pin, which doesn't exist).
    view_bob
        .unpin_record(&record_id)
        .await
        .expect("bob unpin attempt");

    // Alice's pin should still exist - GC should not remove the record.
    let stats = store.garbage_collect().await.expect("run GC");
    assert_eq!(stats.records_removed, 0, "alice's pinned record preserved");
}

#[tokio::test]
async fn test_ttl_overflow_saturates() {
    let (store, _dir) = create_test_store().await;
    let view = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let record_id = view.create_record(genesis).await.expect("create record");

    // Pin with maximum possible TTL - should not overflow.
    view.pin_record(&record_id, Some(u64::MAX))
        .await
        .expect("pin with max TTL should not panic");

    // Record should still be pinned (expires far in future).
    let stats = store.garbage_collect().await.expect("run GC");
    assert_eq!(stats.pins_removed, 0, "max TTL pin should not expire");
    assert_eq!(stats.records_removed, 0);
}
