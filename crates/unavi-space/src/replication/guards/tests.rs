use std::sync::{
    Arc,
    Mutex,
};

use bevy_hsd::document::{
    Hsd,
    HsdDocId,
};
use hsd::{
    attributes::{
        portal::{
            LinkId,
            PortalAttr,
            PortalDestination,
        },
        script::ScriptAttr,
    },
    id::PrimId,
    property::{
        Payload,
        Property,
        value::Value,
    },
    state::HsdState,
};
use iroh_docs::NamespaceSecret;
use unavi_identity::identity::Identity;
use unavi_local::DeviceStorage;
use unavi_policy::{
    Policy,
    rules::Rules,
    trust::{
        Trust,
        TrustTable,
    },
};
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;
use crate::replication::{
    session::compose::stack_on_arrival,
    store::StateFeed,
};

fn identity() -> Identity {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    Identity::new(did, key)
}

fn authored(seed: u8, author: &Identity) -> (DocId, Proven) {
    let ns = NamespaceSecret::from_bytes(&[seed; 32]);
    let proof = Authorship::sign(&ns, author).expect("sign");
    (
        DocId(*ns.id().as_bytes()),
        Proven {
            author: author.did().clone(),
            proof,
        },
    )
}

fn space(seed: &[u8]) -> SpaceId {
    SpaceId(*blake3::hash(seed).as_bytes())
}

/// A distinct, valid endpoint id per seed.
fn peer(seed: u8) -> EndpointId {
    iroh::SecretKey::from_bytes(&[seed; 32]).public()
}

fn prim() -> PrimId {
    PrimId([1; 16])
}

fn state_k() -> PropName {
    hsd::prop_name!("state/k")
}

fn write(name: &PropName, value: &[u8]) -> SessionWrite {
    SessionWrite {
        prim:  prim(),
        name:  name.to_string(),
        value: Some(value.to_vec()),
    }
}

fn world() -> (World, Replicas) {
    let mut world = World::new();
    let policy = Policy::new();
    world.insert_resource(policy.clone());
    let replicas = Replicas::new(policy, Rules::default());
    world.insert_resource(replicas.clone());
    init_indexes(&mut world);
    world.add_observer(stack_on_arrival);
    (world, replicas)
}

/// Spawns `doc` as a live document and applies the layers queued on arrival.
fn spawn_doc(world: &mut World, doc: DocId) -> Arc<Mutex<HsdState>> {
    let state = Arc::new(Mutex::new(HsdState::new()));
    world.spawn((Hsd::shared(Arc::clone(&state)), HsdDocId(doc)));
    world.flush();
    state
}

/// What `name` on the test prim resolves to in `state`.
fn resolved(state: &Mutex<HsdState>, name: &PropName) -> Option<Vec<u8>> {
    state
        .lock()
        .expect("lock")
        .get(prim())
        .and_then(|prim| prim.property(name).and_then(Value::as_attribute).cloned())
        .map(|bytes| bytes.to_vec())
}

/// A replica that has loaded `me` and minted `doc`, so `me` authors it.
fn minted_by(replicas: &Replicas, me: &Identity, doc: DocId) {
    replicas
        .attribution()
        .set_local(me.did().clone(), TrustTable::new(DeviceStorage::memory()));
    replicas.record_minted(doc);
}

fn local(world: &mut World, id: EndpointId, did: &Did) -> Stater {
    Stater {
        entity: local_peer_entity(world),
        id,
        did: Some(did.clone()),
        local: true,
    }
}

fn remote(world: &mut World, id: EndpointId, did: Option<&Did>) -> Stater {
    let entity = claim_remote_peer(world, id, did.cloned(), 0);
    Stater {
        entity,
        id,
        did: did.cloned(),
        local: false,
    }
}

#[test]
fn a_local_pin_broadcasts_and_its_guard_releases_it() {
    let (mut world, replicas) = world();
    let me = identity();
    let (doc, proven) = authored(1, &me);
    let space = space(b"pin-guard");

    let feed = replicas.register_stream(peer(1));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot(_))
    ));

    let stater = local(&mut world, peer(1), me.did());
    assert!(spawn_pin(&mut world, &stater, doc, space, proven));
    assert_eq!(replicas.author_endpoint(space, doc), Some(peer(1)));
    assert!(matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Pin { doc: d, .. }) if d == doc));

    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    world.despawn(tracker);
    assert_eq!(replicas.author_endpoint(space, doc), None);
    assert!(matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Unpin { doc: d }) if d == doc));
    replicas.unregister_stream(feed.token);
}

#[test]
fn a_session_write_composes_into_the_document_it_speaks_for() {
    let (mut world, _) = world();
    let me = identity();
    let space = space(b"compose");
    let state = spawn_doc(&mut world, space.doc());

    let stater = local(&mut world, peer(1), me.did());
    set_session(
        &mut world,
        &stater,
        space.doc(),
        space,
        vec![write(&state_k(), b"v")],
        1,
    )
    .expect("session set");

    assert_eq!(
        resolved(&state, &state_k()).as_deref(),
        Some(&b"v"[..]),
        "what a peer said has to compose in the document it said it about"
    );
}

#[test]
fn a_guests_script_does_not_resolve_while_the_authors_does() {
    let (mut world, replicas) = world();
    let (author, guest) = (identity(), identity());
    let space = space(b"guest-script");
    let doc = space.doc();
    minted_by(&replicas, &author, doc);
    let state = spawn_doc(&mut world, doc);

    let visitor = remote(&mut world, peer(2), Some(guest.did()));
    let batch = vec![write(&ScriptAttr::NAME, &[2; 32]), write(&state_k(), b"v")];
    set_session(&mut world, &visitor, doc, space, batch, 1).expect("session set");
    assert_eq!(resolved(&state, &state_k()).as_deref(), Some(&b"v"[..]));
    assert_eq!(
        resolved(&state, &ScriptAttr::NAME),
        None,
        "a guest must not decide what the author's document runs"
    );

    let owner = local(&mut world, peer(1), author.did());
    set_session(
        &mut world,
        &owner,
        doc,
        space,
        vec![write(&ScriptAttr::NAME, &[1; 32])],
        2,
    )
    .expect("session set");
    assert_eq!(resolved(&state, &ScriptAttr::NAME), Some(vec![1; 32]));
}

#[test]
fn the_authors_opinion_beats_a_guests_on_an_open_key_whatever_the_stamp() {
    let (mut world, replicas) = world();
    let (author, guest) = (identity(), identity());
    let space = space(b"author-wins");

    for (n, guest_first) in [false, true].into_iter().enumerate() {
        let doc = DocId([n as u8 + 1; 32]);
        minted_by(&replicas, &author, doc);
        let state = spawn_doc(&mut world, doc);
        let owner = local(&mut world, peer(1), author.did());
        let visitor = remote(&mut world, peer(2), Some(guest.did()));

        let mut say = |stater: &Stater, value: &[u8], at: u64| {
            set_session(
                &mut world,
                stater,
                doc,
                space,
                vec![write(&state_k(), value)],
                at,
            )
            .expect("session set");
        };
        if guest_first {
            say(&visitor, b"guest", 9);
            say(&owner, b"author", 1);
        } else {
            say(&owner, b"author", 1);
            say(&visitor, b"guest", 9);
        }
        assert_eq!(
            resolved(&state, &state_k()).as_deref(),
            Some(&b"author"[..]),
            "no clock outvotes the author (guest first: {guest_first})"
        );
    }
}

#[test]
fn a_departing_peer_takes_its_session_layer() {
    let (mut world, replicas) = world();
    let space = space(b"layer-departs");
    let doc = space.doc();
    let state = spawn_doc(&mut world, doc);

    let visitor = remote(&mut world, peer(3), None);
    set_session(
        &mut world,
        &visitor,
        doc,
        space,
        vec![write(&state_k(), b"v")],
        1,
    )
    .expect("session set");
    assert_eq!(resolved(&state, &state_k()).as_deref(), Some(&b"v"[..]));
    assert_eq!(replicas.session_peers(doc), vec![peer(3)]);

    world.despawn(visitor.entity);
    assert_eq!(
        resolved(&state, &state_k()),
        None,
        "an opinion goes with the peer that stated it"
    );
    assert!(
        replicas.session_peers(doc).is_empty(),
        "the replicas drop the layer too"
    );
}

#[test]
fn a_layer_goes_with_its_document() {
    let (mut world, replicas) = world();
    let space = space(b"layer-doc");
    let doc = space.doc();
    spawn_doc(&mut world, doc);

    let visitor = remote(&mut world, peer(3), None);
    set_session(
        &mut world,
        &visitor,
        doc,
        space,
        vec![write(&state_k(), b"v")],
        1,
    )
    .expect("session set");
    assert_eq!(replicas.session_peers(doc), vec![peer(3)]);

    let entity = world.resource::<DocIndex>().get(doc).expect("document");
    world.despawn(entity);
    assert!(
        replicas.session_peers(doc).is_empty(),
        "a layer cannot outlive the document it speaks for"
    );
}

#[test]
fn a_layer_stated_before_its_document_arrives_composes_once_it_does() {
    let (mut world, replicas) = world();
    let space = space(b"early-layer");
    let doc = DocId([8; 32]);

    let visitor = remote(&mut world, peer(3), None);
    set_session(
        &mut world,
        &visitor,
        doc,
        space,
        vec![write(&state_k(), b"early")],
        1,
    )
    .expect("session set");
    assert_eq!(replicas.session_peers(doc), vec![peer(3)]);

    let state = spawn_doc(&mut world, doc);
    assert_eq!(
        resolved(&state, &state_k()).as_deref(),
        Some(&b"early"[..]),
        "the document stacks the layers the replicas already hold"
    );
}

#[test]
fn a_departing_peer_takes_its_pins() {
    let (mut world, replicas) = world();
    let alice = identity();
    let (doc, proven) = authored(2, &alice);
    let space = space(b"cascade-peer");

    let stater = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &stater, doc, space, proven));
    assert_eq!(replicas.author_endpoint(space, doc), Some(peer(2)));

    world.despawn(stater.entity);
    assert_eq!(replicas.author_endpoint(space, doc), None);
    assert!(!replicas.has_doc(space, doc));
}

#[test]
fn a_guest_holds_and_reserves_the_next() {
    let (mut world, replicas) = world();
    let alice = identity();
    let (doc, proven) = authored(3, &alice);
    let space = space(b"hold");

    let author_device = remote(&mut world, peer(1), Some(alice.did()));
    assert!(spawn_pin(&mut world, &author_device, doc, space, proven));
    assert_eq!(replicas.holder(space, doc), Some(peer(1)));

    let guest = remote(&mut world, peer(2), None);
    assert!(spawn_hold(&mut world, &guest, doc, space));
    assert_eq!(replicas.holder(space, doc), Some(peer(2)));

    release_hold(&mut world, &guest, doc, Some(peer(3)));
    assert_eq!(replicas.holder(space, doc), Some(peer(1)));

    let other = remote(&mut world, peer(3), None);
    assert!(spawn_hold(&mut world, &other, doc, space));
    assert_eq!(replicas.holder(space, doc), Some(peer(3)));

    world.despawn(other.entity);
    assert_eq!(replicas.holder(space, doc), Some(peer(1)));
}

#[test]
fn a_superseded_stream_leaves_the_peers_state() {
    let (mut world, replicas) = world();
    let alice = identity();
    let (doc, proven) = authored(4, &alice);
    let space = space(b"supersede");

    let e0 = claim_remote_peer(&mut world, peer(2), Some(alice.did().clone()), 0);
    let stater = Stater {
        entity: e0,
        id:     peer(2),
        did:    Some(alice.did().clone()),
        local:  false,
    };
    assert!(spawn_pin(&mut world, &stater, doc, space, proven));

    let e1 = claim_remote_peer(&mut world, peer(2), Some(alice.did().clone()), 1);
    assert_eq!(e0, e1);

    release_remote_peer(&mut world, e0, 0);
    assert_eq!(replicas.author_endpoint(space, doc), Some(peer(2)));

    release_remote_peer(&mut world, e1, 1);
    assert_eq!(replicas.author_endpoint(space, doc), None);
}

#[test]
fn state_for_an_unentered_space_waits_unparented() {
    let (mut world, replicas) = world();
    let alice = identity();
    let (doc, proven) = authored(5, &alice);
    let space = space(b"unentered");

    let stater = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &stater, doc, space, proven));
    assert_eq!(replicas.author_endpoint(space, doc), Some(peer(2)));

    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    assert!(world.get::<ChildOf>(tracker).is_none());
    assert_eq!(
        world.get::<PinnedDoc>(tracker).map(|d| d.space),
        Some(space)
    );
}

/// Rules under which an anonymous peer, the rung a test world with no
/// [`SpaceView`](crate::authority::SpaceView) judges everyone by, may open
/// portals.
fn portal_rules() -> Rules {
    let guest = Rules::default().grant(Trust::Guest).clone();
    Rules::default().with(Trust::Anonymous, guest)
}

/// Hands the next message on `feed` to `world` as if `from` sent it.
fn forward(feed: &StateFeed, world: &mut World, from: Entity) {
    let Ok(ReplicationMsg::Session {
        doc,
        space,
        writes,
        at,
    }) = feed.rx.try_recv()
    else {
        panic!("the visitor broadcasts its session batch");
    };
    apply_remote(
        world,
        from,
        Remote::Session {
            doc,
            space,
            writes,
            at,
        },
    );
}

/// Two peers, each with its own world and replica: the visitor that opens a
/// portal into the host's space, and the host who minted the gate there.
/// Whatever the visitor states, both peers compose the same gate.
#[test]
fn a_far_half_written_by_a_visitor_lands_on_the_host_too() {
    let (mut visitor_world, visitor_replicas) = world();
    let (mut host_world, host_replicas) = world();
    let (visitor, host) = (identity(), identity());
    let visitor_id = peer(1);
    let space = space(b"far-half");
    let gate = DocId([4; 32]);

    for world in [&mut visitor_world, &mut host_world] {
        world.insert_resource(portal_rules());
    }
    host_replicas
        .attribution()
        .set_local(host.did().clone(), TrustTable::new(DeviceStorage::memory()));
    visitor_replicas.attribution().set_local(
        visitor.did().clone(),
        TrustTable::new(DeviceStorage::memory()),
    );
    host_replicas.record_minted(gate);
    let visitor_gate = spawn_doc(&mut visitor_world, gate);
    let host_gate = spawn_doc(&mut host_world, gate);

    let feed = visitor_replicas.register_stream(visitor_id);
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot(_))
    ));
    let from_visitor =
        claim_remote_peer(&mut host_world, visitor_id, Some(visitor.did().clone()), 0);

    let far_half = PortalAttr {
        destination: Some(PortalDestination {
            space: [3; 32],
            link:  Some(LinkId([1; 16])),
        }),
        size_x:      2.0,
        size_y:      2.0,
    }
    .encode()
    .expect("encode");
    let opener = local(&mut visitor_world, visitor_id, visitor.did());
    set_session(
        &mut visitor_world,
        &opener,
        gate,
        space,
        vec![write(&PortalAttr::NAME, &far_half)],
        1,
    )
    .expect("pairing stated");
    forward(&feed, &mut host_world, from_visitor);

    for (side, state) in [("visitor", &visitor_gate), ("host", &host_gate)] {
        assert_eq!(
            resolved(state, &PortalAttr::NAME),
            Some(far_half.clone()),
            "the {side} composes the far half"
        );
    }
    assert_eq!(host_replicas.session_peers(gate), vec![visitor_id]);

    set_session(
        &mut visitor_world,
        &opener,
        gate,
        space,
        vec![write(&ScriptAttr::NAME, &[1; 32])],
        2,
    )
    .expect("the visitor may state anything; the mask decides");
    forward(&feed, &mut host_world, from_visitor);

    for (side, state) in [("visitor", &visitor_gate), ("host", &host_gate)] {
        assert_eq!(
            resolved(state, &ScriptAttr::NAME),
            None,
            "a planted script composes on neither side (the {side})"
        );
    }
    visitor_replicas.unregister_stream(feed.token);
}
