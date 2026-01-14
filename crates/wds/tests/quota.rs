use std::time::Duration;

use rstest::rstest;
use rusqlite::params;
use time::OffsetDateTime;
use tracing_test::traced_test;

use crate::common::{DataStoreCtx, ctx};

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_quota_tracks_envelope_size(#[future] ctx: DataStoreCtx) {
    let did_str = ctx.alice.identity().did().to_string();

    // Get initial quota.
    let initial_used: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    // Create a record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Quota should increase.
    let after_create: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(
        after_create > initial_used,
        "quota should increase after create"
    );

    // Add more data.
    let from_vv = doc.oplog_vv();
    doc.get_map("data")
        .insert("key", "value")
        .expect("insert into map");
    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update record");

    // Quota should increase further.
    let after_update: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(
        after_update > after_create,
        "quota should increase after update"
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_quota_exceeded_rejects_upload(#[future] ctx: DataStoreCtx) {
    let did_str = ctx.alice.identity().did().to_string();

    // Set a tiny quota.
    ctx.store
        .db()
        .call(move |conn| {
            conn.execute(
                "UPDATE user_quotas SET quota_bytes = 1 WHERE owner = ?",
                params![&did_str],
            )?;
            Ok(())
        })
        .await
        .expect("update quota");

    // First, pin a new record ID (record doesn't exist yet but pin is needed).
    let nonce = [0u8; 32];
    let fake_id = blake3::hash(&nonce);
    ctx.alice
        .pin_record(fake_id, Duration::from_secs(3600))
        .await
        .expect("pin record");

    // Try to create a record - should fail due to quota.
    let result = ctx.alice.create_record().send().await;
    assert!(result.is_err(), "should fail due to quota exceeded");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_quota_released_on_gc(#[future] ctx: DataStoreCtx) {
    let did_str = ctx.alice.identity().did().to_string();

    // Get initial quota.
    let initial_used: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    // Create a record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let record_id = result.id;
    let id_str = record_id.to_string();

    // Verify quota increased.
    let after_create: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(after_create > initial_used, "quota should increase");

    // Expire the pin.
    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    ctx.store
        .db()
        .call(move |conn| {
            conn.execute(
                "UPDATE record_pins SET expires = ? WHERE record_id = ?",
                params![past, &id_str],
            )?;
            Ok(())
        })
        .await
        .expect("update expires");

    // Run GC.
    ctx.store.run_gc().await.expect("run gc");

    // Quota should be released.
    let after_gc: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(after_gc, initial_used, "quota should be released after gc");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_quota_shared_by_multiple_pinners(#[future] ctx: DataStoreCtx) {
    let alice_did = ctx.alice.identity().did().to_string();
    let bob_did = ctx.bob.identity().did().to_string();

    // Get initial quotas.
    let alice_initial: i64 = ctx
        .store
        .db()
        .call({
            let alice_did = alice_did.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&alice_did],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    let bob_initial: i64 = ctx
        .store
        .db()
        .call({
            let bob_did = bob_did.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&bob_did],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    // Alice creates a record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let record_id = result.id;

    // Alice's quota increased.
    let alice_after: i64 = ctx
        .store
        .db()
        .call({
            let alice_did = alice_did.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&alice_did],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(alice_after > alice_initial, "alice quota should increase");

    // Bob pins the same record.
    ctx.bob
        .pin_record(record_id, Duration::from_secs(3600))
        .await
        .expect("bob pin");

    // Bob's quota should also increase (charged for existing envelopes).
    let bob_after: i64 = ctx
        .store
        .db()
        .call({
            let bob_did = bob_did.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&bob_did],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(
        bob_after > bob_initial,
        "bob quota should increase when pinning"
    );

    // Expire Alice's pin only.
    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    let id_str = record_id.to_string();
    ctx.store
        .db()
        .call({
            let alice_did = alice_did.clone();
            move |conn| {
                conn.execute(
                    "UPDATE record_pins SET expires = ? WHERE record_id = ? AND owner = ?",
                    params![past, &id_str, &alice_did],
                )?;
                Ok(())
            }
        })
        .await
        .expect("update expires");

    // Run GC.
    ctx.store.run_gc().await.expect("run gc");

    // Alice's quota released, but record still exists (Bob's pin).
    let alice_gc: i64 = ctx
        .store
        .db()
        .call({
            let alice_did = alice_did.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&alice_did],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(alice_gc, alice_initial, "alice quota should be released");

    // Record still exists.
    let id_str = record_id.to_string();
    let record_exists: i64 = ctx
        .store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM records WHERE id = ?",
                params![&id_str],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
        .await
        .expect("query");
    assert_eq!(record_exists, 1, "record should still exist (bob's pin)");
}
