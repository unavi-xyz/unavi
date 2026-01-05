use std::time::Duration;

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
async fn test_write_acl_allows_authorized(#[future] ctx: DataStoreCtx) {
    // Alice creates record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Alice adds Bob to write ACL.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.write.push(ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");

    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("alice update acl");

    // Bob modifies the document.
    let from_vv = doc.oplog_vv();
    let data = doc.get_map("data");
    data.insert("key", "value").expect("insert");

    // Bob should succeed since he's in write ACL.
    ctx.bob
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("bob update");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_read_only_cannot_write(#[future] ctx: DataStoreCtx) {
    // Alice creates record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Alice adds Bob only to read ACL.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.read.push(ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");

    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("alice update acl");

    // Bob tries to write.
    let from_vv = doc.oplog_vv();
    let data = doc.get_map("data");
    data.insert("key", "value").expect("insert");

    // Bob should fail - read permission doesn't grant write.
    let e = ctx
        .bob
        .update_record(record_id, &doc, from_vv)
        .await
        .expect_err("should error");

    assert_contains(e, "denied");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_acl_modification_requires_manage(#[future] ctx: DataStoreCtx) {
    // Alice creates record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Alice adds Bob to write (but not manage) ACL.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.write.push(ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");

    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("alice update acl");

    // Bob tries to add himself to manage.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.manage.push(ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");

    // Should fail - write permission doesn't grant ACL modification.
    let e = ctx
        .bob
        .update_record(record_id, &doc, from_vv)
        .await
        .expect_err("should error");

    assert_contains(e, "denied");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_manager_can_modify_acl(#[future] ctx: DataStoreCtx) {
    // Alice creates record (she has manage).
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Alice adds Bob to write ACL.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.write.push(ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");

    // Should succeed - manager can modify ACL.
    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("alice update acl");

    // Verify Bob is now in write ACL.
    let acl = Acl::load(&doc).expect("load acl");
    assert!(acl.can_write(ctx.bob.identity().did()));
}
