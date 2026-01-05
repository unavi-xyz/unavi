use std::time::Duration;

use loro::LoroValue;
use rstest::rstest;
use tracing_test::traced_test;
use wds::record::acl::Acl;

use crate::common::{DataStoreCtx, assert_contains, ctx};

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_read_own_record(#[future] ctx: DataStoreCtx) {
    // Alice creates a record with some data.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Add some data.
    let from_vv = doc.oplog_vv();
    doc.get_map("data")
        .insert("key", "value")
        .expect("insert into map");
    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update record");

    // Read the record back.
    let read_doc = ctx
        .alice
        .read(record_id)
        .send()
        .await
        .expect("read record");

    // Verify data is present.
    let value = read_doc.get_map("data").get_deep_value();
    let LoroValue::Map(map) = value else {
        panic!("expected map");
    };
    assert_eq!(map.get("key"), Some(&LoroValue::String("value".into())));
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_read_not_found(#[future] ctx: DataStoreCtx) {
    let fake_id = blake3::hash(b"nonexistent");

    let e = ctx
        .alice
        .read(fake_id)
        .send()
        .await
        .expect_err("should error");

    assert_contains(e, "not found");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_read_access_denied(#[future] ctx: DataStoreCtx) {
    // Alice creates a private record (default ACL has no read for Bob).
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let record_id = result.id;

    // Bob tries to read - should fail.
    let e = ctx
        .bob
        .read(record_id)
        .send()
        .await
        .expect_err("should error");

    assert_contains(e, "denied");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_read_with_read_permission(#[future] ctx: DataStoreCtx) {
    // Alice creates a record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Grant Bob read access.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.read.push(ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");
    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update acl");

    // Bob can now read.
    let read_doc = ctx.bob.read(record_id).send().await.expect("read record");

    // Verify doc was read correctly.
    let read_acl = Acl::load(&read_doc).expect("load acl");
    assert!(read_acl.read.contains(ctx.bob.identity().did()));
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_read_with_write_permission(#[future] ctx: DataStoreCtx) {
    // Alice creates a record and grants Bob write access.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.write.push(ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");
    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update acl");

    // Bob can read (write implies read).
    ctx.bob.read(record_id).send().await.expect("read record");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_read_multiple_updates(#[future] ctx: DataStoreCtx) {
    // Alice creates a record and makes several updates.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    for i in 0..5 {
        let from_vv = doc.oplog_vv();
        doc.get_map("data")
            .insert(&format!("key_{i}"), i)
            .expect("insert into map");
        ctx.alice
            .update_record(record_id, &doc, from_vv)
            .await
            .expect("update record");
    }

    // Read and verify all updates are present.
    let read_doc = ctx
        .alice
        .read(record_id)
        .send()
        .await
        .expect("read record");

    let value = read_doc.get_map("data").get_deep_value();
    let LoroValue::Map(map) = value else {
        panic!("expected map");
    };
    for i in 0..5 {
        assert_eq!(
            map.get(&format!("key_{i}")),
            Some(&LoroValue::I64(i))
        );
    }
}
