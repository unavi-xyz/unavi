use std::time::Duration;

use loro::LoroDoc;
use rstest::rstest;
use rusqlite::{OptionalExtension, params};
use tracing_test::traced_test;
use wds::{record::envelope::Envelope, signed_bytes::Signable};
use wired_schemas::surg::{Acl, Record};

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
    let record_id_result: Option<String> = ctx
        .store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT id FROM records WHERE id = ?",
                params![&record_id_str],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
        })
        .await
        .expect("fetch record");

    assert_eq!(record_id_result, Some(record_id.to_string()));
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
    let count: i64 = ctx
        .store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM envelopes WHERE record_id = ?",
                params![&record_id_str],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
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
    let count: i64 = ctx
        .store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM envelopes WHERE record_id = ?",
                params![&record_id_str],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
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
    ctx.store
        .db()
        .call(move |conn| {
            conn.execute(
                "UPDATE user_quotas SET quota_bytes = 0 WHERE owner = ?",
                params![&did_str],
            )?;
            Ok(())
        })
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

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_create_record_mismatched_author(#[future] ctx: DataStoreCtx) {
    // Alice is the record creator, but Bob signs the envelope.
    // This should be rejected - first envelope author must match creator.
    let alice_did = ctx.alice.identity().did().clone();
    let bob_did = ctx.bob.identity().did().clone();

    let doc = LoroDoc::new();

    // Create record with Alice as creator.
    let record = Record::new(alice_did);
    record.save(&doc).expect("save record");

    // Set up ACL with Bob having all permissions (this doesn't matter since
    // the first envelope author validation happens before ACL checks).
    let mut acl = Acl::default();
    acl.add_manager(bob_did.clone());
    acl.add_writer(bob_did.clone());
    acl.add_reader(bob_did.clone());
    acl.save(&doc).expect("save acl");

    let record_id = record.id().expect("record id");

    // Create envelope with Bob as author (mismatched from record creator).
    let envelope = Envelope::all_updates(bob_did, &doc).expect("envelope");
    let signed = envelope
        .sign(ctx.bob.identity().signing_key())
        .expect("sign");

    // Pin record as Bob (pinning doesn't check creator).
    ctx.bob
        .pin_record(record_id, Duration::from_secs(60))
        .await
        .expect("pin record");

    // Upload should fail - envelope author (Bob) != record creator (Alice).
    let e = ctx
        .bob
        .upload_envelope(record_id, &signed)
        .await
        .expect_err("should error");

    assert_contains(e, "denied");
}
