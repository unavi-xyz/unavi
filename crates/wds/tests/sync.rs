use std::time::Duration;

use rstest::rstest;
use rusqlite::params;
use tracing_test::traced_test;
use wds::record::{acl::Acl, schema::SCHEMA_HOME};

use crate::common::{LocalStoreCtx, MultiStoreCtx, multi_ctx, multi_ctx_local};

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

/// Tests sync using did:key with `set_user_identity` (embedded WDS auth pattern).
#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_sync_with_user_identity(#[future] multi_ctx_local: LocalStoreCtx) {
    // Alice creates a record on her store.
    let result = multi_ctx_local
        .alice_ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record on alice's store");
    let (record_id, doc) = (result.id, result.doc);

    // Add some data.
    let from_vv = doc.oplog_vv();

    doc.get_map("data")
        .insert("key", "synced_via_user_identity")
        .expect("insert into map");

    // Grant Bob read access so he can sync and read.
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.read
        .push(multi_ctx_local.bob_ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");

    multi_ctx_local
        .alice_ctx
        .alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update record");

    // Pin the record on Bob's store (required before sync).
    multi_ctx_local
        .bob_ctx
        .bob
        .pin_record(record_id, Duration::from_secs(3600))
        .await
        .expect("pin record on bob's store");

    // Bob syncs from Alice's store using his store's user identity for auth.
    multi_ctx_local
        .bob_ctx
        .bob
        .sync(record_id, multi_ctx_local.alice_ctx.store.endpoint().addr())
        .await
        .expect("sync from alice's store to bob's store");

    // Bob can now read the record on his store.
    let read_doc = multi_ctx_local
        .bob_ctx
        .bob
        .read(record_id)
        .send()
        .await
        .expect("read record on bob's store");

    // Verify data is present.
    let value = read_doc.get_map("data").get_deep_value();
    let loro::LoroValue::Map(map) = value else {
        panic!("expected map");
    };
    assert_eq!(
        map.get("key"),
        Some(&loro::LoroValue::String("synced_via_user_identity".into()))
    );
}

/// Tests that blob dependencies (schemas) are synced when a record uses a custom schema.
#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_sync_transfers_blob_dependencies(#[future] multi_ctx: MultiStoreCtx) {
    // Upload SCHEMA_HOME to Rome only (not to Carthage).
    multi_ctx
        .rome
        .store
        .blobs()
        .blobs()
        .add_slice(&SCHEMA_HOME.bytes)
        .await
        .expect("add schema to Rome");

    // Verify Carthage does NOT have the schema blob.
    let iroh_hash: iroh_blobs::Hash = SCHEMA_HOME.hash.into();
    assert!(
        !multi_ctx
            .carthage
            .store
            .blobs()
            .blobs()
            .has(iroh_hash)
            .await
            .expect("check blob exists"),
        "Carthage should not have schema blob yet"
    );

    // Alice creates a record on Rome with SCHEMA_HOME.
    let result = multi_ctx
        .rome
        .alice
        .create_record()
        .add_schema(&*SCHEMA_HOME, |doc| {
            doc.get_map("home").insert("space", "test_space")?;
            Ok(())
        })
        .expect("add schema")
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

    // Bob on Carthage syncs from Rome.
    // This should transfer the SCHEMA_HOME blob as a dependency.
    multi_ctx
        .carthage
        .bob
        .sync(record_id, multi_ctx.rome.store.endpoint().addr())
        .await
        .expect("sync from Rome to Carthage");

    // Verify Carthage now has the schema blob.
    assert!(
        multi_ctx
            .carthage
            .store
            .blobs()
            .blobs()
            .has(iroh_hash)
            .await
            .expect("check blob exists"),
        "Carthage should have schema blob after sync"
    );

    // Verify the blob dependency is registered for the record.
    let record_id_str = record_id.to_string();
    let hash_str = SCHEMA_HOME.hash.to_string();
    let dep_exists: i64 = multi_ctx
        .carthage
        .store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM record_blob_deps WHERE record_id = ? AND blob_hash = ?)",
                params![&record_id_str, &hash_str],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
        .await
        .expect("query blob deps");
    assert!(
        dep_exists != 0,
        "blob dependency should be registered on Carthage"
    );

    // Verify Bob can read the record on Carthage.
    let read_doc = multi_ctx
        .carthage
        .bob
        .read(record_id)
        .send()
        .await
        .expect("read record on Carthage");

    let value = read_doc.get_map("home").get_deep_value();
    let loro::LoroValue::Map(map) = value else {
        panic!("expected map");
    };
    assert_eq!(
        map.get("space"),
        Some(&loro::LoroValue::String("test_space".into()))
    );
}
