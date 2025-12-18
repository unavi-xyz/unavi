mod common;

use std::str::FromStr;

use common::{DID_ALICE, DID_BOB, create_test_store, create_test_view};
use iroh::{EndpointId, SecretKey};
use wired_data_store::{Genesis, SyncEventType, SyncPeer};
use xdid::core::did::Did;

fn make_test_endpoint_id(seed: u8) -> EndpointId {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    SecretKey::from_bytes(&bytes).public()
}

#[tokio::test]
async fn test_add_and_list_sync_peers() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");
    view.pin_record(record_id, None).await.expect("pin record");

    let peer1 = SyncPeer::new(make_test_endpoint_id(1));
    let peer2 = SyncPeer::new(make_test_endpoint_id(2));

    view.add_sync_peer(record_id, &peer1)
        .await
        .expect("add peer1");
    view.add_sync_peer(record_id, &peer2)
        .await
        .expect("add peer2");

    let sync_peers = view.list_sync_peers(record_id).await.expect("list peers");

    assert_eq!(sync_peers.len(), 2, "should have 2 peers");
    assert!(sync_peers.contains(&peer1), "should contain peer1");
    assert!(sync_peers.contains(&peer2), "should contain peer2");
}

#[tokio::test]
async fn test_remove_sync_peer() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");
    view.pin_record(record_id, None).await.expect("pin record");

    let peer = SyncPeer::new(make_test_endpoint_id(1));

    view.add_sync_peer(record_id, &peer)
        .await
        .expect("add peer");
    view.remove_sync_peer(record_id, &peer)
        .await
        .expect("remove peer");

    let peers = view.list_sync_peers(record_id).await.expect("list peers");

    assert_eq!(peers.len(), 0, "should have no peers after removal");
}

#[tokio::test]
async fn test_sync_peers_cascade_delete_on_unpin() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");
    view.pin_record(record_id, None).await.expect("pin record");

    let peer = SyncPeer::new(make_test_endpoint_id(1));

    view.add_sync_peer(record_id, &peer)
        .await
        .expect("add peer");

    view.unpin_record(record_id).await.expect("unpin record");

    let result = view.list_sync_peers(record_id).await;
    assert!(
        result.as_ref().map_or(true, std::vec::Vec::is_empty),
        "sync peers should be cleaned up when pin removed"
    );
}

#[tokio::test]
async fn test_sync_peers_cascade_delete_on_gc() {
    let (store, _dir) = create_test_store().await;
    let view = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");
    view.pin_record(record_id, Some(1))
        .await
        .expect("pin record with 1 second expiry");

    let peer = SyncPeer::new(make_test_endpoint_id(1));
    view.add_sync_peer(record_id, &peer)
        .await
        .expect("add peer");

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let stats = store.garbage_collect().await.expect("run GC");

    assert_eq!(stats.pins_removed, 1, "should remove expired pin");
    assert_eq!(stats.records_removed, 1, "should remove unpinned record");
}

#[tokio::test]
async fn test_set_sync_peers_atomic() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");
    view.pin_record(record_id, None).await.expect("pin record");

    let peer1 = SyncPeer::new(make_test_endpoint_id(1));
    let peer2 = SyncPeer::new(make_test_endpoint_id(2));
    let peer3 = SyncPeer::new(make_test_endpoint_id(3));

    view.add_sync_peer(record_id, &peer1)
        .await
        .expect("add peer1");
    view.add_sync_peer(record_id, &peer2)
        .await
        .expect("add peer2");

    let new_peers = vec![peer2.clone(), peer3.clone()];
    view.set_sync_peers(record_id, &new_peers)
        .await
        .expect("set peers atomically");

    let sync_peers = view.list_sync_peers(record_id).await.expect("list peers");

    assert_eq!(sync_peers.len(), 2, "should have exactly 2 peers");
    assert!(!sync_peers.contains(&peer1), "should not contain peer1");
    assert!(sync_peers.contains(&peer2), "should contain peer2");
    assert!(sync_peers.contains(&peer3), "should contain peer3");
}

#[tokio::test]
async fn test_set_sync_peers_empty() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");
    view.pin_record(record_id, None).await.expect("pin record");

    let peer = SyncPeer::new(make_test_endpoint_id(1));
    view.add_sync_peer(record_id, &peer)
        .await
        .expect("add peer");

    view.set_sync_peers(record_id, &[])
        .await
        .expect("clear peers");

    let peers = view.list_sync_peers(record_id).await.expect("list peers");

    assert_eq!(peers.len(), 0, "should have no peers");
}

#[tokio::test]
async fn test_sync_peer_from_bytes() {
    let bytes = [42u8; 32];
    let peer = SyncPeer::from_bytes(&bytes).expect("parse from bytes");

    assert_eq!(peer.as_bytes(), &bytes, "should round-trip correctly");
}

#[tokio::test]
async fn test_sync_event_on_create() {
    let (store, _dir) = create_test_store().await;
    let view = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));

    let sync_rx = store.sync_events();

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");

    let event = sync_rx
        .try_recv()
        .expect("should receive sync event on create");

    assert_eq!(
        event.record_id, record_id,
        "event should have correct record_id"
    );
    assert_eq!(
        event.owner_did.to_string(),
        DID_ALICE,
        "event should have correct owner_did"
    );
    assert!(
        matches!(event.event_type, SyncEventType::Created),
        "event should be Created type"
    );
}

#[tokio::test]
async fn test_sync_event_on_delete() {
    let (store, _dir) = create_test_store().await;
    let view = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));

    let sync_rx = store.sync_events();

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");

    sync_rx.try_recv().expect("drain create event");

    view.delete_record(record_id).await.expect("delete record");

    let event = sync_rx
        .try_recv()
        .expect("should receive sync event on delete");

    assert_eq!(
        event.record_id, record_id,
        "event should have correct record_id"
    );
    assert_eq!(
        event.owner_did.to_string(),
        DID_ALICE,
        "event should have correct owner_did"
    );
    assert!(
        matches!(event.event_type, SyncEventType::Deleted),
        "event should be Deleted type"
    );
}

#[tokio::test]
async fn test_multiple_users_sync_same_record() {
    let (store, _dir) = create_test_store().await;
    let view_alice = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));
    let view_bob = store.view_for_user(Did::from_str(DID_BOB).expect("parse DID"));

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view_alice
        .create_record(genesis)
        .await
        .expect("create record");

    view_alice
        .pin_record(record_id, None)
        .await
        .expect("alice pins");
    view_bob
        .pin_record(record_id, None)
        .await
        .expect("bob pins");

    let peer_alice = SyncPeer::new(make_test_endpoint_id(1));
    let peer_bob = SyncPeer::new(make_test_endpoint_id(2));

    view_alice
        .add_sync_peer(record_id, &peer_alice)
        .await
        .expect("alice adds peer");
    view_bob
        .add_sync_peer(record_id, &peer_bob)
        .await
        .expect("bob adds peer");

    let alice_peers = view_alice
        .list_sync_peers(record_id)
        .await
        .expect("list alice peers");
    let bob_peers = view_bob
        .list_sync_peers(record_id)
        .await
        .expect("list bob peers");

    assert_eq!(alice_peers.len(), 1, "alice should see 1 peer");
    assert_eq!(bob_peers.len(), 1, "bob should see 1 peer");
    assert!(alice_peers.contains(&peer_alice), "alice sees her peer");
    assert!(bob_peers.contains(&peer_bob), "bob sees his peer");
}
