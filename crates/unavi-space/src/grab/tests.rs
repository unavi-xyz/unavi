use std::sync::Arc;

use bevy::ecs::system::RunSystemOnce;
use hsd::state::HsdState;
use iroh::{
    EndpointId,
    SecretKey,
};
use unavi_identity::{
    auth::Bindings,
    authorship::Authorship,
    identity::Identity,
    resolver::Resolver,
};
use unavi_local::DeviceStorage;
use unavi_policy::{
    Policy,
    rules::Rules,
    trust::TrustTable,
};
use unavi_registry::client::Followed;
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;
use crate::{
    identity::LocalIdentity,
    replication::{
        record::{
            PinRecord,
            Record,
        },
        store::Proven,
    },
    test_store::{
        block_on,
        store,
    },
};

fn identity() -> Identity {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    Identity::new(did, key)
}

fn peer(seed: u8) -> EndpointId {
    SecretKey::from_bytes(&[seed; 32]).public()
}

/// An App wired with just the hold markers, a real HSD document this node
/// authors, and a [`SpaceView`] for its own identity.
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
    let replicas = Replicas::new(policy.clone(), Rules::default());
    let record = Record::sign(
        &write_key,
        PinRecord {
            doc,
            space,
            at: 1,
            pinned: true,
        },
    )
    .expect("sign pin record");
    let proven = Proven {
        author: alice.did().clone(),
        proof,
    };
    replicas
        .add_pin(other_device, Some(alice.did()), proven, record)
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
        .init_resource::<Holds>();
    crate::replication::guards::init_indexes(app.world_mut());

    let async_world = app.world().resource::<bevy_async::AsyncWorld>().clone();
    let trust = TrustTable::new(DeviceStorage::memory());
    let view = SpaceView::new(
        policy,
        replicas,
        local_identity,
        me,
        trust,
        Rules::default(),
        async_world,
    );
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

/// Runs one reconcile, then drains the async commands [`SpaceView::local`]'s
/// hold calls queue, without waiting for a frame boundary.
fn pump(app: &mut App) {
    app.world_mut()
        .run_system_once(reconcile_holds)
        .expect("run the hold reconcile");
    bevy_async::commands::pump(app.world_mut(), web_time::Instant::now());
}

fn grab(fixture: &mut Fixture) {
    fixture
        .app
        .world_mut()
        .entity_mut(fixture.object)
        .insert(Held {
            pointer:       fixture.pointer,
            reach:         1.0,
            offset_tra:    Vec3::ZERO,
            offset_rot:    Quat::IDENTITY,
            prior_gravity: None,
        });
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

    grab(&mut fixture);
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

    grab(&mut fixture);
    pump(&mut fixture.app);
    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        Some(fixture.me),
        "setup: the grab did not take the hold"
    );

    fixture
        .app
        .world_mut()
        .entity_mut(fixture.object)
        .remove::<Held>();
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

    grab(&mut fixture);
    pump(&mut fixture.app);
    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        Some(fixture.me),
        "setup: the grab did not take the hold"
    );

    // A scene unload, or a script despawning what it grabbed.
    fixture.app.world_mut().despawn(fixture.object);
    pump(&mut fixture.app);

    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        Some(fixture.other_device),
        "a release after the entity despawned could not resolve what hold to drop"
    );
}

#[test]
fn a_grab_whose_document_arrives_late_takes_its_hold() {
    let mut fixture = fixture();
    let tracked = fixture.app.world().resource::<Replicas>().clone();
    fixture
        .app
        .insert_resource(Replicas::new(Policy::new(), Rules::default()));

    grab(&mut fixture);
    pump(&mut fixture.app);
    assert_eq!(
        tracked.holder(fixture.space, fixture.doc),
        Some(fixture.other_device),
        "setup: a hold was taken before the document was tracked"
    );

    fixture.app.insert_resource(tracked.clone());
    pump(&mut fixture.app);

    assert_eq!(
        tracked.holder(fixture.space, fixture.doc),
        Some(fixture.me),
        "a grab still in progress never took the hold once its document was tracked"
    );
}

#[test]
fn grabbing_an_untracked_document_takes_no_hold() {
    let mut fixture = fixture();
    // The hold can only be taken once the publish path has pinned the doc;
    // starting from a fresh `Replicas` leaves it untracked.
    let empty = Replicas::new(Policy::new(), Rules::default());
    fixture.app.insert_resource(empty);

    grab(&mut fixture);
    pump(&mut fixture.app);

    let replicas = fixture.app.world().resource::<Replicas>().clone();
    assert_eq!(
        replicas.holder(fixture.space, fixture.doc),
        None,
        "a grab took a hold on a document nothing had pinned yet"
    );
}
