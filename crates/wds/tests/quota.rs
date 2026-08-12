use std::time::Duration;

use rand::RngCore;
use rstest::rstest;
use rusqlite::params;
use time::OffsetDateTime;
use tracing_test::traced_test;
use wds::DataStore;

use crate::common::{
    DataStoreCtx,
    ctx,
};

mod common;

async fn bytes_used(store: &DataStore, did: &str) -> i64 {
    let did = did.to_string();
    store
        .db()
        .call(move |conn| {
            let used: Option<i64> = conn
                .query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did],
                    |row| row.get(0),
                )
                .ok();
            Ok(used.unwrap_or(0))
        })
        .await
        .expect("query bytes_used")
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_upload_increases_quota(#[future] ctx: DataStoreCtx) {
    let did = ctx.alice.identity().did().to_string();
    assert_eq!(bytes_used(&ctx.store, &did).await, 0);

    let mut bytes = vec![0u8; 4096];
    rand::rng().fill_bytes(&mut bytes);
    ctx.alice
        .upload_blob(bytes.into())
        .await
        .expect("upload blob");

    assert_eq!(bytes_used(&ctx.store, &did).await, 4096);
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_quota_exceeded_rejects_upload(#[future] ctx: DataStoreCtx) {
    let did = ctx.alice.identity().did().to_string();

    ctx.store
        .db()
        .call({
            let did = did.clone();
            move |conn| {
                conn.execute(
                    "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 8)
                     ON CONFLICT(owner) DO UPDATE SET quota_bytes = 8, bytes_used = 0",
                    params![&did],
                )?;
                Ok(())
            }
        })
        .await
        .expect("seed quota");

    let bytes = vec![0u8; 4096];
    let result = ctx.alice.upload_blob(bytes.into()).await;
    assert!(result.is_err(), "upload should be rejected over quota");
    assert_eq!(bytes_used(&ctx.store, &did).await, 0);
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_quota_released_on_gc(#[future] ctx: DataStoreCtx) {
    let did = ctx.alice.identity().did().to_string();

    let mut bytes = vec![0u8; 2048];
    rand::rng().fill_bytes(&mut bytes);
    ctx.alice
        .upload_blob(bytes.into())
        .await
        .expect("upload blob");
    assert_eq!(bytes_used(&ctx.store, &did).await, 2048);

    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    ctx.store
        .db()
        .call({
            let did = did.clone();
            move |conn| {
                conn.execute(
                    "UPDATE blob_pins SET expires = ? WHERE owner = ?",
                    params![past, &did],
                )?;
                Ok(())
            }
        })
        .await
        .expect("expire pin");

    ctx.store.run_gc().await.expect("run gc");

    assert_eq!(bytes_used(&ctx.store, &did).await, 0);
}

/// Re-uploading identical bytes must not charge twice: the pin is a single
/// row, so a second charge could never be released.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_reupload_does_not_double_charge(#[future] ctx: DataStoreCtx) {
    let did = ctx.alice.identity().did().to_string();

    let mut bytes = vec![0u8; 4096];
    rand::rng().fill_bytes(&mut bytes);

    for _ in 0..3 {
        ctx.alice
            .upload_blob(bytes.clone().into())
            .await
            .expect("upload blob");
    }

    assert_eq!(bytes_used(&ctx.store, &did).await, 4096);
}

/// The cap is applied while the body streams in, so an over-quota upload is
/// refused rather than landing on disk and being rejected afterwards.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_upload_past_quota_is_refused_midstream(#[future] ctx: DataStoreCtx) {
    let did = ctx.alice.identity().did().to_string();

    ctx.store
        .db()
        .call({
            let did = did.clone();
            move |conn| {
                conn.execute(
                    "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 1024)
                     ON CONFLICT(owner) DO UPDATE SET quota_bytes = 1024, bytes_used = 0",
                    params![&did],
                )?;
                Ok(())
            }
        })
        .await
        .expect("seed quota");

    let mut bytes = vec![0u8; 64 * 1024];
    rand::rng().fill_bytes(&mut bytes);

    assert!(ctx.alice.upload_blob(bytes.into()).await.is_err());
    assert_eq!(bytes_used(&ctx.store, &did).await, 0);
}
