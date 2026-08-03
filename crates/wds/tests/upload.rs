use std::time::Duration;

use rand::RngCore;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::{
    DataStoreCtx,
    ctx,
};

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_upload_then_exists(#[future] ctx: DataStoreCtx) {
    let mut bytes = vec![0u8; 2048];
    rand::rng().fill_bytes(&mut bytes);

    let hash = ctx
        .alice
        .upload_blob(bytes.into())
        .await
        .expect("upload blob");

    assert!(
        ctx.alice.blob_exists(hash).await.expect("blob exists"),
        "uploaded blob should exist"
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_pin_missing_blob_rejected(#[future] ctx: DataStoreCtx) {
    let missing = blake3::hash(b"never uploaded");
    let result = ctx.alice.pin_blob(missing, Duration::from_hours(1)).await;
    assert!(result.is_err(), "pinning a missing blob should fail");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_pin_extends_existing_blob(#[future] ctx: DataStoreCtx) {
    let mut bytes = vec![0u8; 512];
    rand::rng().fill_bytes(&mut bytes);
    let hash = ctx
        .alice
        .upload_blob(bytes.into())
        .await
        .expect("upload blob");

    ctx.alice
        .pin_blob(hash, Duration::from_hours(2))
        .await
        .expect("pin existing blob");
}
