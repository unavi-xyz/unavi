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
        gossip::unix_secs,
        HeardPresence,
        Peer,
        resync_on_space_added,
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
        .add_observer(resync_on_space_added)
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

/// A state stream toward the peer under test, its opening snapshot read.
fn feed_to_peer(app: &App) -> StateFeed {
    let me = SecretKey::from_bytes(&[3; 32]).public();
    let feed = app
        .world()
        .resource::<Replicas>()
        .register_stream(me, peer_id());
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));
    feed
}

fn resynced(feed: &StateFeed) -> bool {
    matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Snapshot { .. }))
}

#[test]
fn a_peer_arriving_in_a_space_is_sent_a_fresh_snapshot() {
    let mut app = app();
    app.world_mut().spawn(Space(A));
    app.world_mut().spawn(Space(B));
    let feed = feed_to_peer(&app);
    let now = unix_secs();

    present_at(&mut app, A, now);
    assert!(resynced(&feed), "first sight of the peer");

    present_at(&mut app, A, now + 1);
    assert!(!resynced(&feed), "a refresh is not an arrival");

    present_at(&mut app, B, now);
    assert!(resynced(&feed), "an arrival in another space");

    hear(&mut app, A, Heard::Left(now + 2));
    assert!(!resynced(&feed), "a leave sends nothing");

    present_at(&mut app, A, now + 3);
    assert!(resynced(&feed), "arriving again after a leave");
}

#[test]
fn loading_a_space_resyncs_the_peers_heard_in_it() {
    let mut app = app();
    let space = app.world_mut().spawn(Space(A)).id();
    present(&mut app, A);
    let feed = feed_to_peer(&app);

    app.world_mut().despawn(space);
    app.update();
    assert!(!resynced(&feed));

    app.world_mut().spawn(Space(B));
    assert!(!resynced(&feed), "the peer is not in B");

    app.world_mut().spawn(Space(A));
    assert!(resynced(&feed), "the peer is still tracked in A");
}
