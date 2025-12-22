use rand::RngCore;
use rstest::rstest;

use crate::common::{DataStoreCtx, ctx};

mod common;

#[rstest]
#[tokio::test]
#[case(1)]
#[case(32)]
#[case(1024)]
#[case(1024 * 1024)]
#[awt]
async fn test_blob_upload(
    #[future] ctx: DataStoreCtx,
    #[case] blob_len: usize,
) -> anyhow::Result<()> {
    let mut bytes = vec![0u8; blob_len];
    rand::rng().fill_bytes(&mut bytes);

    let hash = blake3::hash(&bytes);
    let recieved_hash = ctx.alice.upload_blob(bytes.into()).await?;

    assert_eq!(recieved_hash, hash);

    Ok(())
}
