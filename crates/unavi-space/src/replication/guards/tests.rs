use hsd::id::PrimId;
use iroh_docs::NamespaceSecret;
use unavi_identity::identity::Identity;
use unavi_policy::Policy;
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;

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

/// One opinion on one key, which is all these tests need of a batch.
fn writes() -> Vec<SessionWrite> {
    vec![SessionWrite {
        prim:  prim(),
        name:  "test/k".to_owned(),
        value: Some(b"v".to_vec()),
    }]
}

fn world() -> (World, Replicas) {
    let mut world = World::new();
    let policy = Policy::new();
    world.insert_resource(policy.clone());
    let replicas = Replicas::new(policy);
    world.insert_resource(replicas.clone());
    init_indexes(&mut world);
    world.add_observer(index::index_document);
    world.add_observer(index::unindex_document);
    (world, replicas)
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
    assert_eq!(replicas.owner(space, doc), Some(peer(1)));
    assert!(matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Pin { doc: d, .. }) if d == doc));

    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    world.despawn(tracker);
    assert_eq!(replicas.owner(space, doc), None);
    assert!(matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Unpin { doc: d }) if d == doc));
    replicas.unregister_stream(feed.token);
}

/// The record is where attribution and replication live; the document's
/// session layer is where the value composes. One call has to reach both,
/// or a peer's opinion is replicated and never drawn.
#[test]
fn a_session_write_composes_into_the_document_it_speaks_for() {
    let (mut world, _) = world();
    let me = identity();
    let space = space(b"compose");

    world.spawn(Space(space));
    let state = Arc::new(Mutex::new(HsdState::new()));
    world.spawn((
        Hsd(Arc::clone(&state)),
        bevy_hsd::document::HsdDocId(space.doc()),
    ));

    let stater = local(&mut world, peer(1), me.did());
    set_session(&mut world, &stater, space.doc(), space, writes(), 1).expect("session set");

    let composed = state.lock().expect("lock").get(prim()).and_then(|p| {
        p.property(&hsd::prop_name!("test/k"))
            .and_then(Value::as_attribute)
            .cloned()
    });
    assert_eq!(
        composed.as_deref(),
        Some(&b"v"[..]),
        "what a peer said has to compose in the document it said it about"
    );
}

#[test]
fn a_cell_survives_its_writer_and_goes_with_its_document() {
    let (mut world, replicas) = world();
    let space = space(b"cell-lifetime");
    let space_ent = world.spawn(Space(space)).id();

    let stater = remote(&mut world, peer(3), None);
    set_session(&mut world, &stater, space.doc(), space, writes(), 1).expect("session set");
    let read = |replicas: &Replicas| {
        replicas.session_value(space, space.doc(), prim(), &hsd::prop_name!("test/k"))
    };
    assert_eq!(read(&replicas), Some(b"v".to_vec()));

    world.despawn(stater.entity);
    assert_eq!(
        read(&replicas),
        Some(b"v".to_vec()),
        "an opinion persists after the peer that wrote it goes"
    );

    world.despawn(space_ent);
    assert_eq!(read(&replicas), None);
}

#[test]
fn a_departing_peer_takes_its_pins() {
    let (mut world, replicas) = world();
    let alice = identity();
    let (doc, proven) = authored(2, &alice);
    let space = space(b"cascade-peer");

    let stater = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &stater, doc, space, proven));
    assert_eq!(replicas.owner(space, doc), Some(peer(2)));

    world.despawn(stater.entity);
    assert_eq!(replicas.owner(space, doc), None);
    assert!(!replicas.has_doc(space, doc));
}

#[test]
fn a_hold_is_refused_to_a_stranger_and_granted_by_release() {
    let (mut world, replicas) = world();
    let alice = identity();
    let (doc, proven) = authored(3, &alice);
    let space = space(b"hold");

    let owner = remote(&mut world, peer(1), Some(alice.did()));
    assert!(spawn_pin(&mut world, &owner, doc, space, proven));

    let guest = remote(&mut world, peer(2), None);
    assert!(!spawn_hold(&mut world, &guest, doc, space));
    assert_eq!(replicas.holder(space, doc), Some(peer(1)));

    release_hold(&mut world, &owner, doc, Some(peer(2)));
    assert!(spawn_hold(&mut world, &guest, doc, space));
    assert_eq!(replicas.holder(space, doc), Some(peer(2)));

    world.despawn(guest.entity);
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
    assert_eq!(replicas.owner(space, doc), Some(peer(2)));

    release_remote_peer(&mut world, e1, 1);
    assert_eq!(replicas.owner(space, doc), None);
}

#[test]
fn state_for_an_unentered_space_waits_unparented() {
    let (mut world, replicas) = world();
    let alice = identity();
    let (doc, proven) = authored(5, &alice);
    let space = space(b"unentered");

    let stater = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &stater, doc, space, proven));
    assert_eq!(replicas.owner(space, doc), Some(peer(2)));

    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    assert!(world.get::<ChildOf>(tracker).is_none());
    assert_eq!(
        world.get::<PinnedDoc>(tracker).map(|d| d.space),
        Some(space)
    );
}
