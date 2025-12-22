use std::time::Duration;

use rand::RngCore;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::{DataStoreCtx, ctx};

mod common;

#[rstest]
#[case(1)]
#[case(32)]
#[case(1024)]
#[case(1024 * 1024)]
#[timeout(Duration::from_secs(2))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_blob_upload(#[future] ctx: DataStoreCtx, #[case] blob_len: usize) {
    let mut bytes = vec![0u8; blob_len];
    rand::rng().fill_bytes(&mut bytes);

    let hash = blake3::hash(&bytes);
    let received_hash = ctx
        .alice
        .upload_blob(bytes.into())
        .await
        .expect("upload blob");

    assert_eq!(received_hash, hash);

    drop(ctx);
}
