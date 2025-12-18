mod common;

use std::str::FromStr;

use common::{DID_DAVE, create_test_view};
use wired_data_store::Genesis;
use xdid::core::did::Did;

#[tokio::test]
async fn test_pin_and_unpin_record() {
    let (view, _dir) = create_test_view(DID_DAVE).await;

    let genesis = Genesis::new(Did::from_str(DID_DAVE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");

    view.pin_record(record_id, None).await.expect("pin record");

    view.unpin_record(record_id).await.expect("unpin record");
}

#[tokio::test]
async fn test_pin_with_ttl() {
    let (view, _dir) = create_test_view(DID_DAVE).await;

    let genesis = Genesis::new(Did::from_str(DID_DAVE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");

    view.pin_record(record_id, Some(3600))
        .await
        .expect("pin record with TTL");

    view.unpin_record(record_id).await.expect("unpin record");
}
