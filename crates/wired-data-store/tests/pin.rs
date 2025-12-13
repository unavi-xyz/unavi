mod common;

use std::str::FromStr;

use common::{DID_DAVE, SCHEMA_PIN_TEST, create_test_store};
use wired_data_store::Genesis;
use xdid::core::did::Did;

#[tokio::test]
async fn test_pin_and_unpin_record() {
    let (store, _dir) = create_test_store(DID_DAVE).await;

    let genesis = Genesis::new(Did::from_str(DID_DAVE).expect("parse DID"), SCHEMA_PIN_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");

    store
        .pin_record(&record_id, None)
        .await
        .expect("pin record");

    store.unpin_record(&record_id).await.expect("unpin record");
}

#[tokio::test]
async fn test_pin_with_ttl() {
    let (store, _dir) = create_test_store(DID_DAVE).await;

    let genesis = Genesis::new(Did::from_str(DID_DAVE).expect("parse DID"), SCHEMA_PIN_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");

    store
        .pin_record(&record_id, Some(3600))
        .await
        .expect("pin record with TTL");

    store.unpin_record(&record_id).await.expect("unpin record");
}
