use std::time::Duration;

use rstest::rstest;
use tracing_test::traced_test;

use crate::common::{DataStoreCtx, assert_contains, ctx};

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_create_record(#[future] ctx: DataStoreCtx) {
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let record_id = result.id;

    // Verify record exists in database.
    let record_id_str = record_id.to_string();
    let record = sqlx::query!("SELECT id FROM records WHERE id = ?", record_id_str)
        .fetch_one(ctx.store.db())
        .await
        .expect("fetch record");

    assert_eq!(record.id, Some(record_id_str));
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_update_record(#[future] ctx: DataStoreCtx) {
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    let from_vv = doc.oplog_vv();

    // Modify the document.
    let data = doc.get_map("data");
    data.insert("key", "value").expect("insert");

    // Upload the update.
    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update record");

    // Verify we have 2 envelopes now.
    let record_id_str = record_id.to_string();
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM envelopes WHERE record_id = ?",
        record_id_str
    )
    .fetch_one(ctx.store.db())
    .await
    .expect("count envelopes");

    assert_eq!(count, 2);
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_multiple_updates(#[future] ctx: DataStoreCtx) {
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Apply 5 updates.
    for i in 0i64..5 {
        let from_vv = doc.oplog_vv();

        let data = doc.get_map("data");
        data.insert(&format!("key_{i}"), i).expect("insert");

        ctx.alice
            .update_record(record_id, &doc, from_vv)
            .await
            .expect("update record");
    }

    // Verify we have 6 envelopes (1 create + 5 updates).
    let record_id_str = record_id.to_string();
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM envelopes WHERE record_id = ?",
        record_id_str
    )
    .fetch_one(ctx.store.db())
    .await
    .expect("count envelopes");

    assert_eq!(count, 6);
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_create_record_quota_exceeded(#[future] ctx: DataStoreCtx) {
    // Set alice's quota to 0.
    let did_str = ctx.alice.identity().did().to_string();
    sqlx::query!(
        "UPDATE user_quotas SET quota_bytes = 0 WHERE owner = ?",
        did_str
    )
    .execute(ctx.store.db())
    .await
    .expect("set quota");

    let e = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect_err("should error");

    assert_contains(e, "quota");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_update_record_unauthorized(#[future] ctx: DataStoreCtx) {
    // Alice creates a record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    let from_vv = doc.oplog_vv();

    // Modify the document.
    let data = doc.get_map("data");
    data.insert("key", "value").expect("insert");

    // Bob tries to update - should fail (not in write ACL).
    let e = ctx
        .bob
        .update_record(record_id, &doc, from_vv)
        .await
        .expect_err("should error");

    assert_contains(e, "denied");
}
