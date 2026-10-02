use std::{
    future::Future,
    sync::Arc,
};

use hsd::state::HsdState;
use iroh::{
    Endpoint,
    EndpointId,
    SecretKey,
    endpoint::presets::N0DisableRelay,
};
use iroh_docs::Author;
use unavi_identity::{
    auth::Bindings,
    authorship::Authorship,
    identity::Identity,
    resolver::Resolver,
};
use unavi_local::DeviceStorage;
use unavi_policy::{
    Policy,
    trust::TrustTable,
};
use unavi_registry::client::Followed;
use unavi_store::{
    Store,
    StoreBuilder,
};
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;
use crate::identity::LocalIdentity;

fn identity() -> Identity {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    Identity::new(did, key)
}

fn peer(seed: u8) -> EndpointId {
    SecretKey::from_bytes(&[seed; 32]).public()
}

/// Runs `future` on the task runtime [`bevy_async::task::spawn`] uses, and
/// blocks this thread for the result.
fn block_on<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> T {
    let (tx, rx) = async_channel::bounded(1);
    bevy_async::task::spawn(async move {
        tx.send(future.await).await.expect("send result");
    });
    rx.recv_blocking().expect("task result")
}

fn store() -> Store {
    block_on(async {
        let secret_key = SecretKey::generate();
        let author = Author::from_bytes(&secret_key.to_bytes());
        let endpoint = Endpoint::builder(N0DisableRelay)
            .secret_key(secret_key)
            .bind()
            .await
            .expect("bind endpoint");
        StoreBuilder::new(endpoint, author)
            .build()
            .await
            .expect("build store")
    })
}

/// An App wired with just the two hold observers, a real HSD document this
/// node authors, and a [`SpaceView`] for its own identity.
struct Fixture {
    app:          App,
    pointer:      Entity,
    /// The entity a grab lands on: an HSD doc, child of its space.
    object:       Entity,
    doc:          DocId,
    space:        SpaceId,
    me:           EndpointId,
    /// The author's other device, which pinned the document and so is its
    /// default holder until this node takes hold.
    other_device: EndpointId,
}

fn fixture() -> Fixture {
    let store = store();
    let document = block_on({
        let store = store.clone();
        async move { store.create().await.expect("create document") }
    });
    let doc = DocId(*document.id().as_bytes());

    let alice = identity();
    let write_key = block_on({
        let ns = document.id();
        async move {
            store
                .write_key(ns)
                .await
                .expect("write key lookup")
                .expect("this store minted the document and so holds its write key")
        }
    });
    let proof = Authorship::sign(&write_key, &alice).expect("sign authorship");

    let space = SpaceId(*blake3::hash(b"unavi-grab-hold-test").as_bytes());
    // `me` is a second device of the same author: the pin comes from
    // elsewhere, so the default holder (the author's first pin) and the
    // holder after this device grabs are observably different peers.
    let other_device = peer(1);
    let me = peer(2);

    let policy = Policy::new();
    let replicas = Replicas::new(policy.clone());
    replicas
        .add_pin(other_device, doc, space, alice.did().clone(), proof, 1)
        .expect("this node's own pin of its own document is never refused");

    let local_identity = LocalIdentity {
        identity: Arc::new(alice),
        bindings: Arc::new(Bindings::default()),
        resolver: Arc::new(Resolver::new().expect("resolver")),
        followed: Followed::default(),
    };

    let mut app = App::new();
    app.add_plugins(bevy_async::AsyncPlugin)
        .insert_resource(replicas.clone())
        .init_resource::<TakenHolds>()
        .add_observer(take_hold_on_grab)
        .add_observer(release_hold_on_release);

    let async_world = app.world().resource::<bevy_async::AsyncWorld>().clone();
    let trust = TrustTable::new(DeviceStorage::memory());
    let view = SpaceView::new(policy, replicas, local_identity, me, trust, async_world);
    app.insert_resource(view);

    let space_entity = app.world_mut().spawn(Space(space)).id();
    let object = app
        .world_mut()
        .spawn((
            Hsd::new(HsdState::new()),
            HsdNamespace(document),
            ChildOf(space_entity),
        ))
        .id();
    let pointer = app.world_mut().spawn_empty().id();

    Fixture {
        app,
        pointer,
        object,
        doc,
        space,
        me,
        other_device,
    }
}

/// Drains the async commands [`SpaceView::local`]'s hold calls queue,
/// without waiting for a frame boundary.
fn pump(app: &mut App) {
    bevy_async::commands::pump(app.world_mut(), web_time::Instant::now());
}

#[test]
fn grabbing_ones_own_object_takes_its_hold() {
    let mut fixture = fixture();
    let replicas = fixture.app.world().resource::<Replicas>().clone();
    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        Some(fixture.other_device),
        "setup: the holder was not the pinning device by default"
    );

    fixture.app.world_mut().trigger(Grabbed {
        entity:  fixture.object,
        pointer: fixture.pointer,
    });
    pump(&mut fixture.app);

    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        Some(fixture.me),
        "grabbing an object this node authors did not take its hold"
    );
}

#[test]
fn releasing_drops_the_hold_it_took() {
    let mut fixture = fixture();
    let replicas = fixture.app.world().resource::<Replicas>().clone();

    fixture.app.world_mut().trigger(Grabbed {
        entity:  fixture.object,
        pointer: fixture.pointer,
    });
    pump(&mut fixture.app);
    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        Some(fixture.me),
        "setup: the grab did not take the hold"
    );

    fixture.app.world_mut().trigger(Released {
        entity:  fixture.object,
        pointer: fixture.pointer,
    });
    pump(&mut fixture.app);

    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        Some(fixture.other_device),
        "releasing the grab left this node's hold in place instead of \
         falling back to the document's author"
    );
}

#[test]
fn releasing_after_the_entity_is_gone_still_drops_the_hold() {
    let mut fixture = fixture();
    let replicas = fixture.app.world().resource::<Replicas>().clone();

    fixture.app.world_mut().trigger(Grabbed {
        entity:  fixture.object,
        pointer: fixture.pointer,
    });
    pump(&mut fixture.app);
    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        Some(fixture.me),
        "setup: the grab did not take the hold"
    );

    // The object is gone by the time the release is told about it — a
    // scene unload, or a script despawning what it grabbed. `unavi-grab`
    // fires `Released` from its own `on_held_removed`, which runs while
    // the despawning entity's components are still readable, but this
    // observer must not depend on that: it reads `TakenHolds` instead.
    fixture.app.world_mut().despawn(fixture.object);
    fixture.app.world_mut().trigger(Released {
        entity:  fixture.object,
        pointer: fixture.pointer,
    });
    pump(&mut fixture.app);

    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        Some(fixture.other_device),
        "a release after the entity despawned could not resolve what hold to drop"
    );
}

#[test]
fn grabbing_an_untracked_document_takes_no_hold() {
    let mut fixture = fixture();
    // The hold can only be taken once the publish path has pinned the doc;
    // starting from a fresh `Replicas` leaves it untracked.
    let empty = Replicas::new(Policy::new());
    fixture.app.insert_resource(empty);

    fixture.app.world_mut().trigger(Grabbed {
        entity:  fixture.object,
        pointer: fixture.pointer,
    });
    pump(&mut fixture.app);

    let replicas = fixture.app.world().resource::<Replicas>().clone();
    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        None,
        "a grab took a hold on a document nothing had pinned yet"
    );
}
