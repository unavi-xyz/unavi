use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use iroh::SecretKey;
use unavi_store::Store;

use super::*;
use crate::test_store::{
    block_on,
    store,
};

fn endpoint(seed: u8) -> EndpointAddr {
    EndpointAddr::new(SecretKey::from_bytes(&[seed; 32]).public())
}

fn document(store: &Store) -> Document {
    let store = store.clone();
    block_on(async move { store.create().await.expect("create document") })
}

#[test]
fn a_space_added_before_the_store_exists_starts_reading_once_it_does() {
    let store = store();
    let doc = document(&store);

    let mut app = App::new();
    let space = app.world_mut().spawn(Space(SpaceId::from(doc.id()))).id();

    app.world_mut()
        .run_system_once(start_space_scenes)
        .expect("run the scene start");
    assert!(
        app.world().get::<PendingScene>(space).is_none(),
        "a scene started without a store to read it from"
    );

    app.insert_resource(DataStore(store));
    app.world_mut()
        .run_system_once(start_space_scenes)
        .expect("run the scene start");
    assert!(
        app.world().get::<PendingScene>(space).is_some(),
        "a space added before the store existed never started reading"
    );
}

#[test]
fn a_space_names_each_occupant_once_it_is_heard_whatever_its_feed_state() {
    let store = store();
    let doc = document(&store);
    let id = SpaceId::from(doc.id());

    let mut app = App::new();
    app.insert_resource(DataStore(store));
    let space = app
        .world_mut()
        .spawn((Space(id), SyncPeers(Vec::new())))
        .id();

    let named = |app: &App| {
        app.world()
            .get::<SyncPeers>(space)
            .map(|peers| peers.0.len())
    };
    app.world_mut()
        .run_system_once(name_late_occupants)
        .expect("run the late occupant naming");
    assert_eq!(named(&app), Some(0), "setup: a peer was named unheard");

    app.world_mut()
        .spawn((Peer(endpoint(1)), LastSeenIn::heard_in(id, Duration::ZERO)));
    for _ in 0..2 {
        app.world_mut()
            .run_system_once(name_late_occupants)
            .expect("run the late occupant naming");
    }
    assert_eq!(
        named(&app),
        Some(1),
        "an occupant heard after the join was not named exactly once"
    );

    app.world_mut().spawn((
        Peer(endpoint(2)),
        LastSeenIn::heard_in(SpaceId([9; 32]), Duration::ZERO),
    ));
    app.world_mut()
        .run_system_once(name_late_occupants)
        .expect("run the late occupant naming");
    assert_eq!(
        named(&app),
        Some(1),
        "a peer heard in another space was named"
    );
}
