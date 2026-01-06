use std::time::Duration;

use rstest::rstest;
use tracing_test::traced_test;
use wds::record::acl::Acl;

use crate::common::{MultiStoreCtx, multi_ctx};

mod common;

#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_sync_record_between_stores(#[future] multi_ctx: MultiStoreCtx) {
    // Alice creates a record on Rome.
    let result = multi_ctx
        .rome
        .alice
        .create_record()
        .send()
        .await
        .expect("create record on Rome");
    let (record_id, doc) = (result.id, result.doc);

    // Add some data.
    let from_vv = doc.oplog_vv();
    doc.get_map("data")
        .insert("key", "synced_value")
        .expect("insert into map");
    multi_ctx
        .rome
        .alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update record");

    // Grant Bob read access so he can read on Carthage.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.read
        .push(multi_ctx.carthage.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");
    multi_ctx
        .rome
        .alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update acl");

    // Pin the record on Carthage (required before sync).
    multi_ctx
        .carthage
        .alice
        .pin_record(record_id, Duration::from_secs(3600))
        .await
        .expect("pin record on Carthage");

    // Bob on Carthage syncs from Rome.
    multi_ctx
        .carthage
        .bob
        .sync(record_id, multi_ctx.rome.store.endpoint().addr())
        .await
        .expect("sync from Rome to Carthage");

    // Bob can now read the record on Carthage.
    let read_doc = multi_ctx
        .carthage
        .bob
        .read(record_id)
        .send()
        .await
        .expect("read record on Carthage");

    // Verify data is present.
    let value = read_doc.get_map("data").get_deep_value();
    let loro::LoroValue::Map(map) = value else {
        panic!("expected map");
    };
    assert_eq!(
        map.get("key"),
        Some(&loro::LoroValue::String("synced_value".into()))
    );
}

#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_read_with_sync_from(#[future] multi_ctx: MultiStoreCtx) {
    // Alice creates a record on Rome.
    let result = multi_ctx
        .rome
        .alice
        .create_record()
        .send()
        .await
        .expect("create record on Rome");
    let (record_id, doc) = (result.id, result.doc);

    // Grant Bob read access.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.read
        .push(multi_ctx.carthage.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");
    multi_ctx
        .rome
        .alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update acl");

    // Pin the record on Carthage.
    multi_ctx
        .carthage
        .alice
        .pin_record(record_id, Duration::from_secs(3600))
        .await
        .expect("pin record on Carthage");

    // Bob reads with sync_from - should sync from Rome then read.
    let read_doc = multi_ctx
        .carthage
        .bob
        .read(record_id)
        .sync_from(multi_ctx.rome.store.endpoint().addr())
        .send()
        .await
        .expect("read with sync_from");

    // Verify ACL is correct.
    let acl = Acl::load(&read_doc).expect("load acl");
    assert!(acl.read.contains(multi_ctx.carthage.bob.identity().did()));
}

#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_sync_updates_after_initial_sync(#[future] multi_ctx: MultiStoreCtx) {
    // Alice creates a record on Rome.
    let result = multi_ctx
        .rome
        .alice
        .create_record()
        .send()
        .await
        .expect("create record on Rome");
    let (record_id, doc) = (result.id, result.doc);

    // Grant Bob read access.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.read
        .push(multi_ctx.carthage.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");
    multi_ctx
        .rome
        .alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update acl");

    // Pin and initial sync.
    multi_ctx
        .carthage
        .alice
        .pin_record(record_id, Duration::from_secs(3600))
        .await
        .expect("pin record on Carthage");
    multi_ctx
        .carthage
        .bob
        .sync(record_id, multi_ctx.rome.store.endpoint().addr())
        .await
        .expect("initial sync");

    // Alice makes more updates on Rome.
    let from_vv = doc.oplog_vv();
    doc.get_map("data")
        .insert("update1", "first")
        .expect("insert");
    multi_ctx
        .rome
        .alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update 1");

    let from_vv = doc.oplog_vv();
    doc.get_map("data")
        .insert("update2", "second")
        .expect("insert");
    multi_ctx
        .rome
        .alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update 2");

    // Bob syncs again to get updates.
    multi_ctx
        .carthage
        .bob
        .sync(record_id, multi_ctx.rome.store.endpoint().addr())
        .await
        .expect("sync updates");

    // Verify updates are present.
    let read_doc = multi_ctx
        .carthage
        .bob
        .read(record_id)
        .send()
        .await
        .expect("read after sync");

    let value = read_doc.get_map("data").get_deep_value();
    let loro::LoroValue::Map(map) = value else {
        panic!("expected map");
    };
    assert_eq!(
        map.get("update1"),
        Some(&loro::LoroValue::String("first".into()))
    );
    assert_eq!(
        map.get("update2"),
        Some(&loro::LoroValue::String("second".into()))
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_sync_unpinned_fails(#[future] multi_ctx: MultiStoreCtx) {
    // Alice creates a record on Rome.
    let result = multi_ctx
        .rome
        .alice
        .create_record()
        .send()
        .await
        .expect("create record on Rome");
    let record_id = result.id;

    // Try to sync without pinning first - should fail.
    // The error may be "not pinned" or a connection error depending on timing.
    multi_ctx
        .carthage
        .bob
        .sync(record_id, multi_ctx.rome.store.endpoint().addr())
        .await
        .expect_err("should fail without pinning");
}
