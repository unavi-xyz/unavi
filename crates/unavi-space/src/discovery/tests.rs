use bevy::prelude::*;
use iroh::{
    EndpointAddr,
    EndpointId,
    SecretKey,
};

use crate::{
    avatar_sync::{
        RemoteAgent,
        receive::despawn_departed_agents,
    },
    discovery::{
        Heard,
        HeardPresence,
        Peer,
        gossip::unix_secs,
        track_peers,
    },
    index::Index,
    membership::{
        Space,
        SpaceId,
    },
    replication::{
        Replicas,
        message::ReplicationMsg,
        request::request_on_space_added,
        store::StateFeed,
    },
};

const A: SpaceId = SpaceId([1; 32]);
const B: SpaceId = SpaceId([2; 32]);

fn peer_id() -> EndpointId {
    SecretKey::from_bytes(&[9; 32]).public()
}

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<HeardPresence>()
        .init_resource::<Index<Peer>>()
        .init_resource::<Index<RemoteAgent>>()
        .init_resource::<Index<Space>>()
        .init_resource::<Replicas>()
        .add_observer(request_on_space_added)
        .add_systems(Update, (track_peers, despawn_departed_agents).chain());
    app
}

fn hear(app: &mut App, space: SpaceId, heard: Heard) {
    app.world()
        .resource::<HeardPresence>()
        .inbox()
        .submit((peer_id(), space), heard);
    app.update();
}

fn present_at(app: &mut App, space: SpaceId, issued_at: u64) {
    hear(
        app,
        space,
        Heard::Present(EndpointAddr::new(peer_id()), issued_at),
    );
}

fn present(app: &mut App, space: SpaceId) {
    present_at(app, space, unix_secs());
}

fn count<C: Component>(app: &mut App) -> usize {
    app.world_mut().query::<&C>().iter(app.world()).count()
}

#[test]
fn a_leave_removes_the_space_and_the_avatar_there() {
    let mut app = app();
    let space_a = app.world_mut().spawn(Space(A)).id();
    app.world_mut().spawn(Space(B));

    present(&mut app, A);
    present(&mut app, B);
    app.world_mut()
        .spawn((RemoteAgent(peer_id()), ChildOf(space_a)));
    app.update();
    assert_eq!(1, count::<RemoteAgent>(&mut app));

    hear(&mut app, A, Heard::Left(unix_secs()));

    assert_eq!(0, count::<RemoteAgent>(&mut app), "avatar left in A");
    assert_eq!(1, count::<Peer>(&mut app), "peer is still in B");
}

#[test]
fn a_leave_from_the_last_space_removes_the_peer() {
    let mut app = app();
    app.world_mut().spawn(Space(A));

    present(&mut app, A);
    assert_eq!(1, count::<Peer>(&mut app));

    hear(&mut app, A, Heard::Left(unix_secs()));

    assert_eq!(0, count::<Peer>(&mut app));
}

#[test]
fn an_avatar_in_a_space_the_peer_is_heard_in_stays() {
    let mut app = app();
    let space_a = app.world_mut().spawn(Space(A)).id();

    present(&mut app, A);
    app.world_mut()
        .spawn((RemoteAgent(peer_id()), ChildOf(space_a)));
    app.update();

    assert_eq!(1, count::<RemoteAgent>(&mut app));
}

#[test]
fn a_leave_stamped_before_the_last_presence_is_ignored() {
    let mut app = app();
    app.world_mut().spawn(Space(A));
    let now = unix_secs();

    present_at(&mut app, A, now);
    hear(&mut app, A, Heard::Left(now - 1));

    assert_eq!(1, count::<Peer>(&mut app), "stale leave ignored");

    hear(&mut app, A, Heard::Left(now + 1));

    assert_eq!(0, count::<Peer>(&mut app), "later leave applies");
}

#[test]
fn a_presence_stamped_before_the_last_leave_is_ignored() {
    let mut app = app();
    app.world_mut().spawn(Space(A));
    let now = unix_secs();

    present_at(&mut app, A, now - 2);
    hear(&mut app, A, Heard::Left(now));
    assert_eq!(0, count::<Peer>(&mut app));

    present_at(&mut app, A, now - 1);
    assert_eq!(0, count::<Peer>(&mut app), "delayed presence ignored");

    present_at(&mut app, A, now + 1);
    assert_eq!(1, count::<Peer>(&mut app), "later presence rejoins");
}

/// A state stream toward each of `seeds`, opening snapshots read.
fn feeds_to(app: &App, seeds: &[u8]) -> Vec<StateFeed> {
    let me = SecretKey::from_bytes(&[3; 32]).public();
    seeds
        .iter()
        .map(|seed| {
            let remote = SecretKey::from_bytes(&[*seed; 32]).public();
            let feed = app
                .world()
                .resource::<Replicas>()
                .register_stream(me, remote);
            assert!(matches!(
                feed.rx.try_recv(),
                Ok(ReplicationMsg::Snapshot { .. })
            ));
            feed
        })
        .collect()
}

fn present_seed(app: &mut App, seed: u8, space: SpaceId) {
    let id = SecretKey::from_bytes(&[seed; 32]).public();
    app.world().resource::<HeardPresence>().inbox().submit(
        (id, space),
        Heard::Present(EndpointAddr::new(id), unix_secs()),
    );
    app.update();
}

fn requested(feed: &StateFeed) -> Option<SpaceId> {
    match feed.rx.try_recv() {
        Ok(ReplicationMsg::Request { space }) => Some(space),
        _ => None,
    }
}

#[test]
fn instancing_a_space_asks_each_peer_heard_in_it_once() {
    let mut app = app();
    present_seed(&mut app, 21, A);
    present_seed(&mut app, 22, A);
    present_seed(&mut app, 23, B);
    let feeds = feeds_to(&app, &[21, 22, 23]);

    app.world_mut().spawn(Space(A));
    app.update();

    assert_eq!(requested(&feeds[0]), Some(A));
    assert_eq!(requested(&feeds[1]), Some(A));
    assert_eq!(requested(&feeds[2]), None, "heard in B, not A");
    for feed in &feeds {
        assert!(feed.rx.try_recv().is_err(), "one request each");
    }
}

#[test]
fn instancing_a_space_again_asks_again() {
    let mut app = app();
    present_seed(&mut app, 21, A);
    let feeds = feeds_to(&app, &[21]);

    let space = app.world_mut().spawn(Space(A)).id();
    assert_eq!(requested(&feeds[0]), Some(A));

    app.world_mut().despawn(space);
    app.world_mut().spawn(Space(A));
    assert_eq!(requested(&feeds[0]), Some(A));
}

#[test]
fn a_peer_heard_after_the_space_loaded_is_not_asked() {
    let mut app = app();
    app.world_mut().spawn(Space(A));
    let feeds = feeds_to(&app, &[21]);

    present_seed(&mut app, 21, A);

    assert_eq!(
        requested(&feeds[0]),
        None,
        "its opening snapshot covered it"
    );
}
