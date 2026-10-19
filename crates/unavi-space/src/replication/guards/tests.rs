use std::{
    sync::{
        Arc,
        Mutex,
    },
    time::Duration,
};

use bevy::ecs::system::RunSystemOnce;
use bevy_hsd::{
    document::{
        Hsd,
        HsdDocId,
    },
    loaded::HsdLoaded,
};
use hsd::{
    attributes::{
        parent::ParentAttr,
        portal::{
            LinkId,
            PortalAttr,
            PortalDestination,
        },
        script::ScriptAttr,
    },
    id::PrimId,
    key,
    property::{
        Payload,
        Property,
        name::PropName,
        value::Value,
    },
    state::{
        HsdState,
        entry::Entry,
    },
};
use iroh::EndpointAddr;
use iroh_docs::NamespaceSecret;
use n0_future::time::Instant;
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
    trust::{
        Trust,
        TrustTable,
    },
};
use unavi_registry::client::Followed as FollowedSet;
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;
use crate::{
    authority::SpaceView,
    discovery::{
        LastSeenIn,
        Peer,
    },
    membership::{
        forget_space,
        register_space,
    },
    pinned::{
        NoLocalPin,
        reconcile_local_pins,
    },
    replication::{
        guards::{
            layers::{
                LocalWrite,
                local_write,
            },
            pins::withdraw_pin,
        },
        message::{
            Received,
            ReceivedDoc,
            session::{
                ClearedKey,
                LayerState,
                MAX_VALUE_BYTES,
                MAX_WRITES_PER_TICK,
                ReceivedLayer,
                ReceivedTick,
                SessionWrite,
                SignedLayer,
                check_tick,
            },
        },
        request::request_on_space_added,
        session::{
            SessionError,
            WriterId,
            cells::Cells,
            compose::{
                SessionLayerState,
                stack_on_arrival,
            },
        },
        store::StateFeed,
    },
};

fn identity() -> Identity {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    Identity::new(did, key)
}

/// A document `author` wrote in `space`, and the pin that says so.
fn authored(seed: u8, author: &Identity, space: SpaceId) -> (DocId, PinClaim) {
    authored_at(seed, author, space, 1)
}

/// As [`authored`], with the pin record stamped `at`.
fn authored_at(seed: u8, author: &Identity, space: SpaceId, at: u64) -> (DocId, PinClaim) {
    let ns = NamespaceSecret::from_bytes(&[seed; 32]);
    let proof = Authorship::sign(&ns, author).expect("sign");
    let claim = PinClaim {
        proven: Proven {
            author: author.did().clone(),
            proof,
        },
        record: record_of(seed, space, at, true),
    };
    (DocId(*ns.id().as_bytes()), claim)
}

/// The record `authored(seed, ..)`'s namespace signs for its document.
fn record_of(seed: u8, space: SpaceId, at: u64, pinned: bool) -> Record {
    let ns = NamespaceSecret::from_bytes(&[seed; 32]);
    let payload = PinRecord {
        doc: DocId(*ns.id().as_bytes()),
        space,
        at,
        pinned,
    };
    Record::sign(&ns, payload).expect("sign")
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

/// Authors an empty portal frame on the test prim, as a gate's asset does.
fn author_frame(state: &Mutex<HsdState>) {
    let mut state = state.lock().expect("lock");
    state
        .project(&Entry::new(
            key::Key::prop(prim(), &ParentAttr::NAME).to_string(),
            ParentAttr::to_wire(Some(ParentAttr::Root)),
            1,
        ))
        .expect("root");
    let frame = PortalAttr {
        destination: None,
        size_x:      2.0,
        size_y:      2.0,
    }
    .encode()
    .expect("encode");
    state
        .project(&Entry::new(
            key::Key::prop(prim(), &PortalAttr::NAME).to_string(),
            Value::Attribute(frame.into()).encode(),
            1,
        ))
        .expect("frame");
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

fn local(world: &mut World, id: EndpointId, did: &Did) -> Source {
    Source {
        entity: local_peer_entity(world),
        id,
        did: Some(did.clone()),
        local: true,
    }
}

fn remote(world: &mut World, id: EndpointId, did: Option<&Did>) -> Source {
    let entity = claim_remote_peer(world, id, did.cloned(), 0);
    Source {
        entity,
        id,
        did: did.cloned(),
        local: false,
    }
}

/// A peer speaking for `who` about `doc` in `space`.
struct Speaker<'a> {
    source: &'a Source,
    who:    &'a Identity,
    doc:    DocId,
    space:  SpaceId,
}

impl Speaker<'_> {
    fn state(&self, at: u64, root: [u8; 32]) -> SignedLayer {
        SignedLayer::sign(
            LayerState {
                device: self.source.id,
                did: self.who.did().clone(),
                doc: self.doc,
                space: self.space,
                at,
                root,
            },
            self.who.signing_key(),
        )
        .expect("sign")
    }

    /// The tick setting `writes`, stamped `at`.
    fn tick(&self, writes: &[SessionWrite], at: u64) -> ReceivedTick {
        let (writes, clears) = check_tick(writes, &[]).expect("within bounds");
        ReceivedTick {
            writes,
            clears,
            state: self.state(at, [0; 32]),
        }
    }

    /// The whole layer holding `writes`, stamped `at`, as a snapshot states it.
    fn layer(&self, writes: &[SessionWrite], at: u64) -> ReceivedLayer {
        let (writes, _) = check_tick(writes, &[]).expect("within bounds");
        let mut cells = Cells::default();
        for (key, value) in writes {
            cells.set(key, value);
        }
        ReceivedLayer {
            state: self.state(at, cells.root()),
            cells,
        }
    }

    /// States `writes`: through the local write path for the local peer, which
    /// stamps it by its own clock, and as a received tick otherwise.
    fn say(&self, world: &mut World, writes: Vec<SessionWrite>, at: u64) {
        if self.source.local {
            let write = LocalWrite {
                doc: self.doc,
                space: self.space,
                writes,
                clears: Vec::new(),
            };
            local_write(world, self.who, self.source.id, write).expect("local write");
        } else {
            layers::apply_tick(world, self.source, &self.tick(&writes, at));
        }
    }
}

#[test]
fn a_local_pin_broadcasts_and_its_guard_releases_it() {
    let (mut world, replicas) = world();
    let me = identity();
    let space = space(b"pin-guard");
    let (doc, proven) = authored(1, &me, space);

    let feed = replicas.register_stream(peer(1), peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));

    let source = local(&mut world, peer(1), me.did());
    assert!(spawn_pin(&mut world, &source, proven));
    assert_eq!(replicas.author_endpoint(space, doc), Some(peer(1)));
    assert!(
        matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Pin { record, .. }) if record.payload().is_ok_and(|r| r.doc == doc))
    );

    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    world.despawn(tracker);
    assert_eq!(replicas.author_endpoint(space, doc), None);
    assert!(matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Unpin { doc: d, .. }) if d == doc));
    replicas.unregister_stream(feed.token);
}

#[test]
fn a_session_write_composes_into_the_document_it_speaks_for() {
    let (mut world, _) = world();
    let me = identity();
    let space = space(b"compose");
    let state = spawn_doc(&mut world, space.doc());

    let source = local(&mut world, peer(1), me.did());
    Speaker {
        source: &source,
        who: &me,
        doc: space.doc(),
        space,
    }
    .say(&mut world, vec![write(&state_k(), b"v")], 1);

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
    Speaker {
        source: &visitor,
        who: &guest,
        doc,
        space,
    }
    .say(&mut world, batch, 1);
    assert_eq!(resolved(&state, &state_k()).as_deref(), Some(&b"v"[..]));
    assert_eq!(
        resolved(&state, &ScriptAttr::NAME),
        None,
        "a guest must not decide what the author's document runs"
    );

    let owner = local(&mut world, peer(1), author.did());
    Speaker {
        source: &owner,
        who: &author,
        doc,
        space,
    }
    .say(&mut world, vec![write(&ScriptAttr::NAME, &[1; 32])], 2);
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

        let mut speak = |source: &Source, who: &Identity, value: &[u8], at: u64| {
            Speaker {
                source,
                who,
                doc,
                space,
            }
            .say(&mut world, vec![write(&state_k(), value)], at);
        };
        if guest_first {
            speak(&visitor, &guest, b"guest", 9);
            speak(&owner, &author, b"author", 1);
        } else {
            speak(&owner, &author, b"author", 1);
            speak(&visitor, &guest, b"guest", 9);
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

    let guest = identity();
    let visitor = remote(&mut world, peer(3), Some(guest.did()));
    Speaker {
        source: &visitor,
        who: &guest,
        doc,
        space,
    }
    .say(&mut world, vec![write(&state_k(), b"v")], 1);
    assert_eq!(resolved(&state, &state_k()).as_deref(), Some(&b"v"[..]));
    assert_eq!(
        replicas.session_writers(doc),
        vec![WriterId::new(peer(3), guest.did())]
    );

    world.despawn(visitor.entity);
    assert_eq!(
        resolved(&state, &state_k()),
        None,
        "an opinion goes with the peer that stated it"
    );
    assert!(
        replicas.session_writers(doc).is_empty(),
        "the replicas drop the layer too"
    );
}

#[test]
fn a_layer_goes_with_its_document() {
    let (mut world, replicas) = world();
    let space = space(b"layer-doc");
    let doc = space.doc();
    spawn_doc(&mut world, doc);

    let guest = identity();
    let visitor = remote(&mut world, peer(3), Some(guest.did()));
    Speaker {
        source: &visitor,
        who: &guest,
        doc,
        space,
    }
    .say(&mut world, vec![write(&state_k(), b"v")], 1);
    assert_eq!(
        replicas.session_writers(doc),
        vec![WriterId::new(peer(3), guest.did())]
    );

    let entity = world.resource::<DocIndex>().get(doc).expect("document");
    world.despawn(entity);
    assert!(
        replicas.session_writers(doc).is_empty(),
        "a layer cannot outlive the document it speaks for"
    );
}

#[test]
fn a_layer_stated_before_its_document_arrives_composes_once_it_does() {
    let (mut world, replicas) = world();
    let space = space(b"early-layer");
    let doc = DocId([8; 32]);

    let guest = identity();
    let visitor = remote(&mut world, peer(3), Some(guest.did()));
    Speaker {
        source: &visitor,
        who: &guest,
        doc,
        space,
    }
    .say(&mut world, vec![write(&state_k(), b"early")], 1);
    assert_eq!(
        replicas.session_writers(doc),
        vec![WriterId::new(peer(3), guest.did())]
    );

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
    let space = space(b"cascade-peer");
    let (doc, proven) = authored(2, &alice, space);

    let source = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &source, proven));
    assert_eq!(replicas.author_endpoint(space, doc), Some(peer(2)));

    world.despawn(source.entity);
    assert_eq!(replicas.author_endpoint(space, doc), None);
    assert!(!replicas.has_doc(space, doc));
}

/// Two viewers hearing the same two grabs in opposite orders name the same
/// holder, and fall back to the author endpoint together.
#[test]
fn holds_heard_in_any_order_resolve_one_holder() {
    let alice = identity();
    let space = space(b"hold");
    let viewer = |first: (EndpointId, u64), second: (EndpointId, u64)| {
        let (mut world, replicas) = world();
        let (doc, proven) = authored(3, &alice, space);
        let author_device = remote(&mut world, peer(1), Some(alice.did()));
        assert!(spawn_pin(&mut world, &author_device, proven));
        assert_eq!(replicas.holder(space, doc), Some(peer(1)));
        for (id, at) in [first, second] {
            let grabber = remote(&mut world, id, None);
            spawn_hold(&mut world, &grabber, HoldClaim { doc, space, at });
        }
        (world, replicas, doc)
    };
    let (early, late) = ((peer(2), 10), (peer(3), 20));
    let (mut world_a, replicas_a, doc) = viewer(early, late);
    let (_world_b, replicas_b, _) = viewer(late, early);
    assert_eq!(replicas_a.holder(space, doc), Some(peer(3)));
    assert_eq!(replicas_b.holder(space, doc), Some(peer(3)));

    let (_tie_world, tie, _) = viewer((peer(2), 30), (peer(3), 30));
    assert_eq!(
        tie.holder(space, doc),
        Some(peer(2).max(peer(3))),
        "a tied stamp goes to the greater endpoint"
    );

    let late_entity = index::lookup::<RemotePeer>(&world_a, peer(3)).expect("late grabber");
    world_a.despawn(late_entity);
    assert_eq!(replicas_a.holder(space, doc), Some(peer(2)));
    let early_entity = index::lookup::<RemotePeer>(&world_a, peer(2)).expect("early grabber");
    let early_source = Source {
        entity: early_entity,
        id:     peer(2),
        did:    None,
        local:  false,
    };
    release_hold(&mut world_a, &early_source, doc);
    assert_eq!(replicas_a.holder(space, doc), Some(peer(1)));
}

/// The author endpoint is the author device whose pin record is stamped
/// earliest, whichever pin a viewer heard first.
#[test]
fn author_pins_heard_in_any_order_resolve_one_author_endpoint() {
    let alice = identity();
    let space = space(b"author-order");
    let viewer = |order: [(EndpointId, u64); 2]| {
        let (mut world, replicas) = world();
        let mut doc = None;
        for (id, at) in order {
            let (id_doc, claim) = authored_at(4, &alice, space, at);
            let device = remote(&mut world, id, Some(alice.did()));
            assert!(spawn_pin(&mut world, &device, claim));
            doc = Some(id_doc);
        }
        let doc = doc.expect("pinned");
        replicas.author_endpoint(space, doc)
    };
    let (phone, laptop) = ((peer(1), 5), (peer(2), 9));
    assert_eq!(viewer([phone, laptop]), Some(peer(1)));
    assert_eq!(viewer([laptop, phone]), Some(peer(1)));
}

#[test]
fn a_superseded_stream_leaves_the_peers_state() {
    let (mut world, replicas) = world();
    let alice = identity();
    let space = space(b"supersede");
    let (doc, proven) = authored(4, &alice, space);

    let e0 = claim_remote_peer(&mut world, peer(2), Some(alice.did().clone()), 0);
    let source = Source {
        entity: e0,
        id:     peer(2),
        did:    Some(alice.did().clone()),
        local:  false,
    };
    assert!(spawn_pin(&mut world, &source, proven));

    let e1 = claim_remote_peer(&mut world, peer(2), Some(alice.did().clone()), 1);
    assert_eq!(e0, e1);

    release_remote_peer(&mut world, e0, 0);
    assert_eq!(replicas.author_endpoint(space, doc), Some(peer(2)));

    release_remote_peer(&mut world, e1, 1);
    assert_eq!(replicas.author_endpoint(space, doc), None);
}

#[test]
fn a_snapshot_keeps_what_it_restates_and_drops_the_rest() {
    let (mut world, replicas) = world();
    let alice = identity();
    let space = space(b"restate");
    let (kept, kept_claim) = authored(6, &alice, space);
    let (dropped, dropped_claim) = authored(7, &alice, space);
    let device = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &device, kept_claim));
    assert!(spawn_pin(&mut world, &device, dropped_claim));
    spawn_hold(
        &mut world,
        &device,
        HoldClaim {
            doc: kept,
            space,
            at: 3,
        },
    );
    let guard = |world: &World, doc: DocId| {
        world.get::<PeerGuards>(device.entity).and_then(|guards| {
            guards
                .iter()
                .find(|e| world.get::<PinState>(*e).is_some_and(|pin| pin.doc == doc))
        })
    };
    let before = guard(&world, kept).expect("pin guard");

    let (_, restated) = authored(6, &alice, space);
    apply_remote(
        &mut world,
        device.entity,
        Remote::Snapshot {
            scope: None,
            docs:  vec![RemoteDoc {
                doc: kept,
                space,
                pin: Some(restated),
                hold: Some(3),
                withdrawal: None,
                layers: Vec::new(),
            }],
        },
    );

    assert_eq!(
        guard(&world, kept),
        Some(before),
        "a restated pin keeps its guard rather than being released and taken again"
    );
    assert!(guard(&world, dropped).is_none());
    assert!(!replicas.pins_by(peer(2), dropped));
    assert_eq!(replicas.holder(space, kept), Some(peer(2)));
}

#[test]
fn state_for_an_unentered_space_waits_unparented() {
    let (mut world, replicas) = world();
    let alice = identity();
    let space = space(b"unentered");
    let (doc, proven) = authored(5, &alice, space);

    let source = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &source, proven));
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
    let Ok(msg @ ReplicationMsg::Session(_)) = feed.rx.try_recv() else {
        panic!("the visitor broadcasts its session tick");
    };
    let Ok(Received::Session(tick)) = msg.validate(clock::current_micros()) else {
        panic!("a broadcast tick passes validation");
    };
    apply_remote(world, from, Remote::Session(tick));
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
    author_frame(&visitor_gate);
    author_frame(&host_gate);

    let feed = visitor_replicas.register_stream(visitor_id, peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
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
    let opener = |writes: Vec<SessionWrite>| LocalWrite {
        doc: gate,
        space,
        writes,
        clears: Vec::new(),
    };
    local_write(
        &mut visitor_world,
        &visitor,
        visitor_id,
        opener(vec![write(&PortalAttr::NAME, &far_half)]),
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
    assert_eq!(
        host_replicas.session_writers(gate),
        vec![WriterId::new(visitor_id, visitor.did())]
    );

    local_write(
        &mut visitor_world,
        &visitor,
        visitor_id,
        opener(vec![write(&ScriptAttr::NAME, &[1; 32])]),
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

#[test]
fn a_guest_cannot_redirect_a_portal_the_document_authors() {
    let (mut world, replicas) = world();
    world.insert_resource(portal_rules());
    let author = identity();
    let space = space(b"authored-portal");
    let doc = space.doc();
    minted_by(&replicas, &author, doc);
    let state = spawn_doc(&mut world, doc);

    let authored = PortalAttr {
        destination: Some(PortalDestination {
            space: [8; 32],
            link:  None,
        }),
        size_x:      2.0,
        size_y:      2.0,
    }
    .encode()
    .expect("encode");
    {
        let mut state = state.lock().expect("lock");
        state
            .project(&Entry::new(
                key::Key::prop(prim(), &ParentAttr::NAME).to_string(),
                ParentAttr::to_wire(Some(ParentAttr::Root)),
                1,
            ))
            .expect("root");
        state
            .project(&Entry::new(
                key::Key::prop(prim(), &PortalAttr::NAME).to_string(),
                Value::Attribute(authored.clone().into()).encode(),
                1,
            ))
            .expect("portal");
    }

    let redirect = PortalAttr {
        destination: Some(PortalDestination {
            space: [9; 32],
            link:  Some(LinkId([2; 16])),
        }),
        size_x:      2.0,
        size_y:      2.0,
    }
    .encode()
    .expect("encode");
    let guest = identity();
    let visitor = remote(&mut world, peer(2), Some(guest.did()));
    Speaker {
        source: &visitor,
        who: &guest,
        doc,
        space,
    }
    .say(&mut world, vec![write(&PortalAttr::NAME, &redirect)], 2);
    assert_eq!(
        resolved(&state, &PortalAttr::NAME),
        Some(authored),
        "a pairing fills an empty frame and never redirects an authored portal"
    );
}

#[test]
fn a_cleared_opinion_leaves_the_document_and_every_replica() {
    let (mut world, replicas) = world();
    let me = identity();
    let space = space(b"cleared");
    let doc = space.doc();
    let state = spawn_doc(&mut world, doc);
    let feed = replicas.register_stream(peer(1), peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));

    let tick = |writes: Vec<SessionWrite>, clears: Vec<ClearedKey>| LocalWrite {
        doc,
        space,
        writes,
        clears,
    };
    local_write(
        &mut world,
        &me,
        peer(1),
        tick(vec![write(&state_k(), b"v")], Vec::new()),
    )
    .expect("session set");
    assert_eq!(resolved(&state, &state_k()).as_deref(), Some(&b"v"[..]));
    assert!(matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Session(_))));

    let cleared = vec![ClearedKey {
        prim: prim(),
        name: state_k().to_string(),
    }];
    local_write(&mut world, &me, peer(1), tick(Vec::new(), cleared)).expect("cleared");
    assert_eq!(resolved(&state, &state_k()), None);
    assert_eq!(
        replicas.session_opinions(WriterId::new(peer(1), me.did()), doc),
        Vec::new()
    );
    assert!(
        matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Session(_))),
        "a commit's clear has to reach every peer still drawing the old opinion"
    );
    replicas.unregister_stream(feed.token);
}

/// A world whose local peer is `me` at `peer(1)`, with a feed watching its
/// broadcasts.
fn local_world(me: &Identity) -> (World, Replicas, StateFeed) {
    let (mut world, replicas) = world();
    world.insert_resource(view_of(
        &replicas,
        me,
        TrustTable::new(DeviceStorage::memory()),
    ));
    world.insert_resource(LocalIdentity {
        identity: Arc::new(Identity::new(me.did().clone(), P256KeyPair::generate())),
        bindings: Arc::new(Bindings::default()),
        resolver: Arc::new(Resolver::new().expect("resolver")),
        followed: FollowedSet::default(),
    });
    let feed = replicas.register_stream(peer(1), peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));
    (world, replicas, feed)
}

/// Despawns every session layer guard.
fn release_layers(world: &mut World) {
    let guards = world
        .query::<(Entity, &SessionLayerState)>()
        .iter(world)
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    for guard in guards {
        world.despawn(guard);
    }
}

#[test]
fn releasing_the_local_layer_retracts_it_for_peers() {
    let me = identity();
    let (mut world, replicas, feed) = local_world(&me);
    let space = space(b"retract-local");
    let doc = space.doc();
    spawn_doc(&mut world, doc);
    local_write(
        &mut world,
        &me,
        peer(1),
        LocalWrite {
            doc,
            space,
            writes: vec![write(&state_k(), b"v")],
            clears: Vec::new(),
        },
    )
    .expect("session set");
    assert!(matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Session(_))));

    release_layers(&mut world);

    let Ok(ReplicationMsg::Session(tick)) = feed.rx.try_recv() else {
        panic!("the clear reaches peers");
    };
    assert!(tick.writes.is_empty());
    assert_eq!(tick.clears.len(), 1);
    replicas.unregister_stream(feed.token);
}

#[test]
fn releasing_a_remote_layer_sends_nothing() {
    let me = identity();
    let (mut world, replicas, feed) = local_world(&me);
    let space = space(b"retract-remote");
    let doc = space.doc();
    spawn_doc(&mut world, doc);
    let guest = identity();
    let visitor = remote(&mut world, peer(2), Some(guest.did()));
    Speaker {
        source: &visitor,
        who: &guest,
        doc,
        space,
    }
    .say(&mut world, vec![write(&state_k(), b"v")], 2);
    while feed.rx.try_recv().is_ok() {}

    release_layers(&mut world);

    assert!(
        feed.rx.try_recv().is_err(),
        "a remote writer's layer going says nothing about what it stated"
    );
    replicas.unregister_stream(feed.token);
}

#[test]
fn a_local_tick_too_large_for_a_frame_is_refused_whole() {
    let (mut world, replicas) = world();
    let me = identity();
    let space = space(b"split");
    let doc = space.doc();
    let feed = replicas.register_stream(peer(1), peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));

    let tick = |writes| LocalWrite {
        doc,
        space,
        writes,
        clears: Vec::new(),
    };
    let by_size = (0..20)
        .map(|i| write(&format!("state/w{i}").parse().expect("name"), &[0; 15_000]))
        .collect();
    let by_count = (0..=MAX_WRITES_PER_TICK)
        .map(|i| write(&format!("state/w{i}").parse().expect("name"), &[1]))
        .collect();
    for writes in [by_size, by_count] {
        assert_eq!(
            local_write(&mut world, &me, peer(1), tick(writes)).err(),
            Some(SessionError::TooLarge)
        );
    }
    assert!(feed.rx.try_recv().is_err(), "nothing is sent");
    assert!(
        replicas.session_writers(doc).is_empty(),
        "nothing is applied"
    );
    replicas.unregister_stream(feed.token);
}

#[test]
fn a_value_past_the_cap_is_refused_locally() {
    let (mut world, replicas) = world();
    let me = identity();
    let space = space(b"cap");
    let doc = space.doc();
    let tick = LocalWrite {
        doc,
        space,
        writes: vec![write(&state_k(), &vec![0; MAX_VALUE_BYTES + 1])],
        clears: Vec::new(),
    };
    assert_eq!(
        local_write(&mut world, &me, peer(1), tick).err(),
        Some(SessionError::TooLarge)
    );
    assert!(
        replicas.session_writers(doc).is_empty(),
        "a refused write stacks no layer"
    );
}

#[test]
fn a_relayed_layer_is_ignored_while_its_writer_is_connected() {
    let (mut world, replicas) = world();
    let (alice, bob) = (identity(), identity());
    let space = space(b"connected");
    let doc = space.doc();
    let writer = remote(&mut world, peer(2), Some(alice.did()));
    let relayer = remote(&mut world, peer(4), Some(bob.did()));

    let speaker = Speaker {
        source: &writer,
        who: &alice,
        doc,
        space,
    };
    let remote_doc = RemoteDoc {
        doc,
        space,
        pin: None,
        hold: None,
        withdrawal: None,
        layers: vec![speaker.layer(&[write(&state_k(), b"v")], 1)],
    };
    apply_remote(
        &mut world,
        relayer.entity,
        Remote::Snapshot {
            scope: None,
            docs:  vec![remote_doc],
        },
    );

    assert!(
        replicas.session_writers(doc).is_empty(),
        "the connected writer has not stated it"
    );
}

/// A document `alice` authors, in a world viewed from `peer(1)`, with one of
/// her devices present at `peer(2)` having stated a script and some state.
struct Present {
    world:  World,
    state:  Arc<Mutex<HsdState>>,
    doc:    DocId,
    space:  SpaceId,
    alice:  Identity,
    me:     Identity,
    device: Source,
}

impl Present {
    /// Alice's device's layer.
    fn writer(&self) -> WriterId {
        WriterId::new(peer(2), self.alice.did())
    }

    /// Alice's device, speaking about the document.
    fn speaker<'a>(&'a self, source: &'a Source) -> Speaker<'a> {
        Speaker {
            source,
            who: &self.alice,
            doc: self.doc,
            space: self.space,
        }
    }
}

fn alice_is_present(replicas: &Replicas, mut world: World, trust: TrustTable) -> Present {
    let (alice, me, bob) = (identity(), identity(), identity());
    let space = space(b"carry");
    let (doc, proof) = authored(50, &alice, space);
    world.insert_resource(view_of(replicas, &me, trust));
    let state = spawn_doc(&mut world, doc);

    // A pin by someone else keeps the document, and so its author, known once
    // alice's device leaves.
    let relayer = remote(&mut world, peer(3), Some(bob.did()));
    assert!(spawn_pin(&mut world, &relayer, proof));

    let device = remote(&mut world, peer(2), Some(alice.did()));
    let writes = vec![
        write(&ScriptAttr::NAME, &[1; 32]),
        write(&state_k(), b"old"),
    ];
    Speaker {
        source: &device,
        who: &alice,
        doc,
        space,
    }
    .say(&mut world, writes, 1);
    Present {
        world,
        state,
        doc,
        space,
        alice,
        me,
        device,
    }
}

#[test]
fn the_authors_engine_opinion_survives_its_departure_when_the_viewer_adopts_it() {
    let (world, replicas) = world();
    let mut f = alice_is_present(&replicas, world, TrustTable::new(DeviceStorage::memory()));
    assert_eq!(resolved(&f.state, &ScriptAttr::NAME), Some(vec![1; 32]));

    release_remote_peer(&mut f.world, f.device.entity, 0);

    assert_eq!(
        resolved(&f.state, &ScriptAttr::NAME),
        Some(vec![1; 32]),
        "a carried layer is still masked as the author's"
    );
    assert_eq!(replicas.session_sources(f.writer(), f.doc), vec![peer(1)]);
}

#[test]
fn a_peer_whose_rung_is_not_adopted_does_not_carry() {
    let (world, replicas) = world();
    let trust = TrustTable::new(DeviceStorage::memory());
    let mut f = alice_is_present(&replicas, world, trust.clone());
    trust
        .set(f.alice.did().clone(), Trust::Anonymous)
        .expect("set");

    release_remote_peer(&mut f.world, f.device.entity, 0);

    assert!(
        replicas.session_writers(f.doc).is_empty(),
        "nothing holds the layer once its writer is gone"
    );
    assert_eq!(resolved(&f.state, &state_k()), None);
}

#[test]
fn a_carried_layer_reaches_a_late_joiner_masked_by_the_writers_standing() {
    let (viewer_world, replicas) = world();
    let mut f = alice_is_present(
        &replicas,
        viewer_world,
        TrustTable::new(DeviceStorage::memory()),
    );
    release_remote_peer(&mut f.world, f.device.entity, 0);

    let feed = replicas.register_stream(peer(1), peer(9));
    let Ok(msg @ ReplicationMsg::Snapshot { .. }) = feed.rx.try_recv() else {
        panic!("a stream opens with a snapshot");
    };
    replicas.unregister_stream(feed.token);
    let Ok(Received::Snapshot { docs, .. }) = msg.validate(clock::current_micros()) else {
        panic!("the carried layer passes validation");
    };

    let (mut joiner_world, joiner_replicas) = world();
    let (joiner, bob) = (identity(), identity());
    let trust = TrustTable::new(DeviceStorage::memory());
    trust.set(f.me.did().clone(), Trust::Blocked).expect("set");
    joiner_world.insert_resource(view_as(&joiner_replicas, &joiner, peer(9), trust));
    let (doc, proof) = authored(50, &f.alice, f.space);
    let joiner_state = spawn_doc(&mut joiner_world, doc);
    let pinner = remote(&mut joiner_world, peer(3), Some(bob.did()));
    assert!(spawn_pin(&mut joiner_world, &pinner, proof));

    let relayer = remote(&mut joiner_world, peer(1), Some(f.me.did()));
    let docs = docs
        .into_iter()
        .map(|d| RemoteDoc {
            doc:        d.doc,
            space:      d.space,
            pin:        None,
            hold:       d.hold,
            withdrawal: d.withdrawal,
            layers:     d.layers,
        })
        .collect();
    apply_remote(
        &mut joiner_world,
        relayer.entity,
        Remote::Snapshot { scope: None, docs },
    );

    assert_eq!(
        joiner_replicas.session_layers(doc),
        vec![(f.writer(), f.alice.did().clone())]
    );
    assert_eq!(
        resolved(&joiner_state, &ScriptAttr::NAME),
        Some(vec![1; 32]),
        "the layer is the author's, whoever relays it"
    );
    assert_eq!(
        joiner_replicas.session_sources(f.writer(), doc),
        vec![peer(1)]
    );
}

#[test]
fn the_writers_return_replaces_the_carried_layer() {
    let (world, replicas) = world();
    let mut f = alice_is_present(&replicas, world, TrustTable::new(DeviceStorage::memory()));
    release_remote_peer(&mut f.world, f.device.entity, 0);
    assert_eq!(replicas.session_sources(f.writer(), f.doc), vec![peer(1)]);

    let back = remote(&mut f.world, peer(2), Some(f.alice.did()));
    let layer = f.speaker(&back).layer(&[write(&state_k(), b"fresh")], 5);
    let doc = RemoteDoc {
        doc:        f.doc,
        space:      f.space,
        pin:        None,
        hold:       None,
        withdrawal: None,
        layers:     vec![layer],
    };
    apply_remote(
        &mut f.world,
        back.entity,
        Remote::Snapshot {
            scope: None,
            docs:  vec![doc],
        },
    );

    assert_eq!(
        replicas.session_sources(f.writer(), f.doc),
        vec![peer(2)],
        "the writer is present, so only its word counts"
    );
    assert_eq!(
        resolved(&f.state, &state_k()).as_deref(),
        Some(&b"fresh"[..])
    );
    assert_eq!(
        resolved(&f.state, &ScriptAttr::NAME),
        None,
        "the snapshot replaces the layer outright"
    );
}

#[test]
fn a_live_tick_from_the_returning_writer_drops_the_carry() {
    let (world, replicas) = world();
    let mut f = alice_is_present(&replicas, world, TrustTable::new(DeviceStorage::memory()));
    release_remote_peer(&mut f.world, f.device.entity, 0);

    let back = remote(&mut f.world, peer(2), Some(f.alice.did()));
    let tick = f.speaker(&back).tick(&[write(&state_k(), b"live")], 5);
    apply_remote(&mut f.world, back.entity, Remote::Session(tick));

    assert_eq!(replicas.session_sources(f.writer(), f.doc), vec![peer(2)]);
    assert_eq!(
        resolved(&f.state, &state_k()).as_deref(),
        Some(&b"live"[..])
    );
}

#[test]
fn a_relayed_layer_is_ignored_while_its_writer_is_present() {
    let (world, replicas) = world();
    let mut f = alice_is_present(&replicas, world, TrustTable::new(DeviceStorage::memory()));

    let relayer = remote(&mut f.world, peer(4), Some(f.me.did()));
    let stale = f
        .speaker(&f.device)
        .layer(&[write(&state_k(), b"stale")], 5);
    let doc = RemoteDoc {
        doc:        f.doc,
        space:      f.space,
        pin:        None,
        hold:       None,
        withdrawal: None,
        layers:     vec![stale],
    };
    apply_remote(
        &mut f.world,
        relayer.entity,
        Remote::Snapshot {
            scope: None,
            docs:  vec![doc],
        },
    );

    assert_eq!(replicas.session_sources(f.writer(), f.doc), vec![peer(2)]);
    assert_eq!(resolved(&f.state, &state_k()).as_deref(), Some(&b"old"[..]));
}

/// A view of `replicas` from `me`'s side, judging DIDs by `trust`.
fn view_of(replicas: &Replicas, me: &Identity, trust: TrustTable) -> SpaceView {
    view_as(replicas, me, peer(1), trust)
}

/// A view of `replicas` from the endpoint `id` of `me`.
fn view_as(replicas: &Replicas, me: &Identity, id: EndpointId, trust: TrustTable) -> SpaceView {
    let mut app = App::new();
    app.add_plugins(bevy_async::AsyncPlugin);
    let async_world = app.world().resource::<bevy_async::AsyncWorld>().clone();
    // Only the DID is read, so a fresh key stands in for the real one.
    let identity = LocalIdentity {
        identity: Arc::new(Identity::new(me.did().clone(), P256KeyPair::generate())),
        bindings: Arc::new(Bindings::default()),
        resolver: Arc::new(Resolver::new().expect("resolver")),
        followed: FollowedSet::default(),
    };
    SpaceView::new(
        Policy::new(),
        replicas.clone(),
        identity,
        id,
        trust,
        Rules::default(),
        async_world,
    )
}

/// Marks `tracker` as a document that finished loading.
fn load(world: &mut World, tracker: Entity) {
    world
        .entity_mut(tracker)
        .insert((Hsd::new(HsdState::new()), HsdLoaded));
}

/// Runs the system that pins loaded documents for the local peer.
fn reconcile(world: &mut World) {
    world
        .run_system_once(reconcile_local_pins)
        .expect("reconcile ran");
}

/// A document authored by `alice`, pinned by a remote author device and by
/// the local peer as a non-author, with a feed watching the local peer's
/// broadcasts.
struct Scenario {
    author: Source,
    doc:    DocId,
    proof:  PinClaim,
    space:  SpaceId,
    feed:   StateFeed,
}

/// Puts the local peer in `space`, as a loaded space entity does.
fn join(replicas: &Replicas, space: SpaceId) {
    replicas
        .attribution()
        .policy()
        .update(space.doc(), |record| record.space = Some(space.doc()));
}

fn doc_of(seed: u8) -> DocId {
    DocId(*NamespaceSecret::from_bytes(&[seed; 32]).id().as_bytes())
}

/// The author's last device withdrawing `doc`, with a record stamped `at`.
fn withdraw(world: &mut World, author: Entity, doc: DocId, space: SpaceId, at: u64) {
    let withdrawal = Some(record_of(40, space, at, false));
    apply_remote(world, author, Remote::Unpin { doc, withdrawal });
}

fn locally_pinned_foreign_doc(
    world: &mut World,
    replicas: &Replicas,
    alice: &Identity,
    me: &Identity,
) -> Scenario {
    let space = space(b"follow");
    join(replicas, space);
    let (doc, proven) = authored(40, alice, space);
    let (_, proof) = authored(40, alice, space);
    let feed = replicas.register_stream(peer(1), peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));

    let author = remote(world, peer(2), Some(alice.did()));
    assert!(spawn_pin(world, &author, proven));
    let viewer = local(world, peer(1), me.did());
    let (_, mine) = authored(40, alice, space);
    assert!(spawn_pin(world, &viewer, mine));
    assert!(!replicas.pins_as_author(peer(1), doc));
    assert!(
        matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Pin { record, .. }) if record.payload().is_ok_and(|r| r.doc == doc))
    );
    Scenario {
        author,
        doc,
        proof,
        space,
        feed,
    }
}

#[test]
fn an_author_unpin_drops_a_local_non_author_pin() {
    let (mut world, replicas) = world();
    let (alice, me) = (identity(), identity());
    let f = locally_pinned_foreign_doc(&mut world, &replicas, &alice, &me);

    withdraw(&mut world, f.author.entity, f.doc, f.space, 2);

    assert!(!replicas.is_pinned(f.doc), "nobody holds it any more");
    let Ok(ReplicationMsg::Unpin { doc, withdrawal }) = f.feed.rx.try_recv() else {
        panic!("the local peer tells everyone it let go");
    };
    assert_eq!(doc, f.doc);
    assert!(
        withdrawal.is_some(),
        "and relays the withdrawal that made it"
    );
    replicas.unregister_stream(f.feed.token);
}

#[test]
fn an_author_unpin_leaves_the_local_pin_while_another_author_device_pins() {
    let (mut world, replicas) = world();
    let (alice, me) = (identity(), identity());
    let f = locally_pinned_foreign_doc(&mut world, &replicas, &alice, &me);
    let second = remote(&mut world, peer(3), Some(alice.did()));
    assert!(spawn_pin(&mut world, &second, f.proof));

    apply_remote(
        &mut world,
        f.author.entity,
        Remote::Unpin {
            doc:        f.doc,
            withdrawal: None,
        },
    );

    assert!(replicas.is_author_pinned(f.doc));
    assert!(replicas.sync_sources(f.doc, peer(9)).contains(&peer(1)));
    assert!(f.feed.rx.try_recv().is_err());
    replicas.unregister_stream(f.feed.token);
}

#[test]
fn a_remote_authors_disconnect_is_not_an_unpin() {
    let (mut world, replicas) = world();
    let (alice, me) = (identity(), identity());
    let f = locally_pinned_foreign_doc(&mut world, &replicas, &alice, &me);

    release_remote_peer(&mut world, f.author.entity, 0);

    assert!(!replicas.is_author_pinned(f.doc));
    assert!(
        replicas.sync_sources(f.doc, peer(9)).contains(&peer(1)),
        "the local pin outlives the author's connection"
    );
    assert!(f.feed.rx.try_recv().is_err(), "nothing is broadcast");
    replicas.unregister_stream(f.feed.token);
}

#[test]
fn a_non_author_pin_keeps_a_doc_loaded_while_the_author_is_adopted() {
    let (mut world, replicas) = world();
    let (alice, me, bob) = (identity(), identity(), identity());
    let space = space(b"loads");
    let (doc, proven) = authored(41, &alice, space);
    let trust = TrustTable::new(DeviceStorage::memory());
    let view = view_of(&replicas, &me, trust.clone());
    assert!(!view.loads(doc), "nothing pins it");

    let relayer = remote(&mut world, peer(3), Some(bob.did()));
    assert!(spawn_pin(&mut world, &relayer, proven));
    assert!(
        view.loads(doc),
        "an adopted author's document outlives its author"
    );

    trust.set(alice.did().clone(), Trust::Blocked).expect("set");
    assert!(
        !view.loads(doc),
        "a blocked author's document needs the author"
    );

    let (_, proof) = authored(41, &alice, space);
    let author = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &author, proof));
    assert!(view.loads(doc), "the author's own pin always loads it");
}

#[test]
fn a_loaded_foreign_doc_is_pinned_locally_until_it_unloads() {
    let (mut world, replicas) = world();
    let (alice, me) = (identity(), identity());
    let space = space(b"pin-on-load");
    let (doc, proven) = authored(42, &alice, space);
    world.insert_resource(view_of(
        &replicas,
        &me,
        TrustTable::new(DeviceStorage::memory()),
    ));
    let feed = replicas.register_stream(peer(1), peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));

    let author = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &author, proven));
    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    load(&mut world, tracker);
    reconcile(&mut world);

    assert!(replicas.sync_sources(doc, peer(9)).contains(&peer(1)));
    assert!(!replicas.pins_as_author(peer(1), doc));
    assert!(
        matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Pin { record, .. }) if record.payload().is_ok_and(|r| r.doc == doc))
    );

    world.despawn(tracker);
    assert!(!replicas.sync_sources(doc, peer(9)).contains(&peer(1)));
    assert!(matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Unpin { doc: d, .. }) if d == doc));
    replicas.unregister_stream(feed.token);
}

#[test]
fn a_loaded_doc_by_an_unadopted_author_is_not_pinned_locally() {
    let (mut world, replicas) = world();
    let (alice, me) = (identity(), identity());
    let space = space(b"pin-on-load-anonymous");
    let (doc, proven) = authored(43, &alice, space);
    let trust = TrustTable::new(DeviceStorage::memory());
    trust
        .set(alice.did().clone(), Trust::Anonymous)
        .expect("set");
    world.insert_resource(view_of(&replicas, &me, trust));

    let author = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &author, proven));
    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    load(&mut world, tracker);
    reconcile(&mut world);

    assert!(!replicas.sync_sources(doc, peer(9)).contains(&peer(1)));
}

#[test]
fn a_withdrawn_doc_is_not_kept_loaded_by_the_remaining_non_author_pins() {
    let (mut world, replicas) = world();
    let (alice, me, bob) = (identity(), identity(), identity());
    let f = locally_pinned_foreign_doc(&mut world, &replicas, &alice, &me);
    let view = view_of(&replicas, &me, TrustTable::new(DeviceStorage::memory()));
    let relayer = remote(&mut world, peer(3), Some(bob.did()));
    assert!(spawn_pin(&mut world, &relayer, f.proof));

    withdraw(&mut world, f.author.entity, f.doc, f.space, 2);

    assert!(replicas.is_withdrawn(f.doc));
    assert!(replicas.is_pinned(f.doc), "the relayer still pins it");
    assert!(!view.loads(f.doc));
    replicas.unregister_stream(f.feed.token);
}

#[test]
fn a_newer_author_pin_restores_a_withdrawn_doc() {
    let (mut world, replicas) = world();
    let (alice, me, bob) = (identity(), identity(), identity());
    let f = locally_pinned_foreign_doc(&mut world, &replicas, &alice, &me);
    let view = view_of(&replicas, &me, TrustTable::new(DeviceStorage::memory()));
    let relayer = remote(&mut world, peer(3), Some(bob.did()));
    assert!(spawn_pin(&mut world, &relayer, f.proof));
    withdraw(&mut world, f.author.entity, f.doc, f.space, 2);
    assert!(replicas.is_withdrawn(f.doc));

    let (_, again) = authored_at(40, &alice, f.space, 3);
    let author = remote(&mut world, peer(5), Some(alice.did()));
    assert!(spawn_pin(&mut world, &author, again));

    assert!(!replicas.is_withdrawn(f.doc));
    assert!(view.loads(f.doc));
    replicas.unregister_stream(f.feed.token);
}

#[test]
fn a_stale_pin_record_does_not_restore_a_withdrawn_doc() {
    let (mut world, replicas) = world();
    let (alice, me, bob) = (identity(), identity(), identity());
    let f = locally_pinned_foreign_doc(&mut world, &replicas, &alice, &me);
    let view = view_of(&replicas, &me, TrustTable::new(DeviceStorage::memory()));
    withdraw(&mut world, f.author.entity, f.doc, f.space, 2);

    let (_, stale) = authored(40, &alice, f.space);
    let late = remote(&mut world, peer(6), Some(bob.did()));
    assert!(spawn_pin(&mut world, &late, stale));

    assert!(replicas.is_pinned(f.doc), "the pin is counted");
    assert!(
        replicas.is_withdrawn(f.doc),
        "the withdrawal is still the newest record"
    );
    assert!(!view.loads(f.doc), "whoever relays the stale pin");
    replicas.unregister_stream(f.feed.token);
}

#[test]
fn a_withdrawal_outlives_the_presence_and_is_listed_in_snapshots() {
    let (mut world, replicas) = world();
    let (alice, me) = (identity(), identity());
    let f = locally_pinned_foreign_doc(&mut world, &replicas, &alice, &me);

    withdraw(&mut world, f.author.entity, f.doc, f.space, 2);

    assert!(!replicas.is_pinned(f.doc), "nobody holds it any more");
    assert!(
        replicas.is_withdrawn(f.doc),
        "carriers keep it to tell late joiners"
    );
    let feed = replicas.register_stream(peer(1), peer(9));
    let Ok(ReplicationMsg::Snapshot { docs, .. }) = feed.rx.try_recv() else {
        panic!("a stream opens with a snapshot");
    };
    let listed = docs
        .iter()
        .find(|d| d.doc == f.doc)
        .expect("a withdrawn doc is listed");
    assert!(listed.pin.is_none());
    let record = listed.withdrawal.as_ref().expect("with its withdrawal");
    assert!(!record.payload().expect("decode").pinned);
    replicas.unregister_stream(feed.token);
    replicas.unregister_stream(f.feed.token);
}

#[test]
fn a_late_joiner_told_only_by_a_carrier_learns_the_withdrawal() {
    let (alice, me, bob) = (identity(), identity(), identity());
    let space = space(b"carrier");
    let (doc, stale) = authored(40, &alice, space);

    let (_carrier_world, carrier) = world();
    join(&carrier, space);
    carrier.observe_record(&record_of(40, space, 2, false), peer(2));
    let feed = carrier.register_stream(peer(3), peer(9));
    let Ok(msg @ ReplicationMsg::Snapshot { .. }) = feed.rx.try_recv() else {
        panic!("a stream opens with a snapshot");
    };
    carrier.unregister_stream(feed.token);
    let Ok(Received::Snapshot { docs, .. }) = msg.validate(clock::current_micros()) else {
        panic!("the snapshot passes validation");
    };

    let (mut world, joiner) = world();
    join(&joiner, space);
    let view = view_of(&joiner, &me, TrustTable::new(DeviceStorage::memory()));
    let from_carrier = remote(&mut world, peer(3), Some(bob.did()));
    let docs = docs
        .into_iter()
        .map(|d| RemoteDoc {
            doc:        d.doc,
            space:      d.space,
            pin:        None,
            hold:       d.hold,
            withdrawal: d.withdrawal,
            layers:     d.layers,
        })
        .collect();
    apply_remote(
        &mut world,
        from_carrier.entity,
        Remote::Snapshot { scope: None, docs },
    );
    assert!(joiner.is_withdrawn(doc), "the carrier told it");

    let pinner = remote(&mut world, peer(4), Some(bob.did()));
    assert!(spawn_pin(&mut world, &pinner, stale));
    assert!(joiner.is_pinned(doc));
    assert!(
        !view.loads(doc),
        "a pin that predates the withdrawal does not load it"
    );
}

#[test]
fn leaving_a_space_sends_a_plain_unpin_and_leaves_no_record() {
    let (mut world, replicas) = world();
    world.add_observer(register_space);
    world.add_observer(forget_space);
    let alice = identity();
    let space = space(b"leave-ordering");
    let (doc, claim) = authored(47, &alice, space);

    let space_entity = world.spawn(Space(space)).id();
    let feed = replicas.register_stream(peer(1), peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));
    let laptop = local(&mut world, peer(1), alice.did());
    assert!(spawn_pin(&mut world, &laptop, claim));
    assert!(matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Pin { .. })));
    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    assert_eq!(
        world.get::<ChildOf>(tracker).map(ChildOf::parent),
        Some(space_entity),
        "the pin hangs under the space"
    );

    world.despawn(space_entity);

    let Ok(ReplicationMsg::Unpin { withdrawal, .. }) = feed.rx.try_recv() else {
        panic!("the pin's guard says it let go");
    };
    assert!(withdrawal.is_none(), "leaving a space withdraws nothing");
    assert!(!replicas.is_withdrawn(doc));
    assert_eq!(
        replicas.newest_record_at(doc),
        None,
        "the space's records go once it is left, after its guards"
    );
    replicas.unregister_stream(feed.token);
}

#[test]
fn withdrawing_signs_past_every_record_and_unpins_with_it() {
    let (mut world, replicas) = world();
    let alice = identity();
    let space = space(b"withdrawal-signing");
    join(&replicas, space);
    let (doc, claim) = authored(48, &alice, space);
    let laptop = local(&mut world, peer(1), alice.did());
    assert!(spawn_pin(&mut world, &laptop, claim));
    let feed = replicas.register_stream(peer(1), peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));

    let ahead = clock::current_micros() + 5_000_000;
    replicas.observe_record(&record_of(48, space, ahead, true), peer(2));
    let key = NamespaceSecret::from_bytes(&[48; 32]);
    withdraw_pin(&mut world, peer(1), doc, &key).expect("withdraw");

    let Ok(ReplicationMsg::Unpin { withdrawal, .. }) = feed.rx.try_recv() else {
        panic!("the unpin goes out");
    };
    let payload = withdrawal
        .expect("the unpin carries the withdrawal")
        .payload()
        .expect("decode");
    assert!(!payload.pinned);
    assert_eq!(
        payload.at,
        ahead + 1,
        "stamped past the newest record known"
    );
    assert!(replicas.is_withdrawn(doc), "kept for late joiners");
    assert!(
        !replicas.pins_by(peer(1), doc),
        "the local pin went with it"
    );

    let absent = doc_of(49);
    withdraw_pin(&mut world, peer(1), absent, &key).expect("nothing to withdraw");
    assert!(
        feed.rx.try_recv().is_err(),
        "a document nobody pins sends nothing"
    );
    replicas.unregister_stream(feed.token);
}

#[test]
fn an_unloaded_authored_document_is_unpinned_not_withdrawn() {
    let (mut world, replicas) = world();
    let alice = identity();
    let space = space(b"unload-is-not-delete");
    join(&replicas, space);
    let (doc, claim) = authored(50, &alice, space);
    let laptop = local(&mut world, peer(1), alice.did());
    assert!(spawn_pin(&mut world, &laptop, claim));
    let feed = replicas.register_stream(peer(1), peer(9));
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot { .. })
    ));

    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    world.despawn(tracker);

    let Ok(ReplicationMsg::Unpin { withdrawal, .. }) = feed.rx.try_recv() else {
        panic!("the pin's guard says it let go");
    };
    assert!(
        withdrawal.is_none(),
        "only deleting a document withdraws it, even while in its space"
    );
    assert!(!replicas.is_withdrawn(doc));
    replicas.unregister_stream(feed.token);
}

#[test]
fn reconcile_does_not_repin_a_withdrawn_doc() {
    let (mut world, replicas) = world();
    let (alice, me, bob) = (identity(), identity(), identity());
    let f = locally_pinned_foreign_doc(&mut world, &replicas, &alice, &me);
    world.insert_resource(view_of(
        &replicas,
        &me,
        TrustTable::new(DeviceStorage::memory()),
    ));
    let relayer = remote(&mut world, peer(3), Some(bob.did()));
    assert!(spawn_pin(&mut world, &relayer, f.proof));
    withdraw(&mut world, f.author.entity, f.doc, f.space, 2);
    let tracker = index::lookup::<PinnedDoc>(&world, f.doc).expect("tracker");
    load(&mut world, tracker);

    reconcile(&mut world);

    assert!(!replicas.pins_by(peer(1), f.doc));
    replicas.unregister_stream(f.feed.token);
}

#[test]
fn reconcile_repins_a_doc_the_local_peer_authors() {
    let (mut world, replicas) = world();
    let (me, bob) = (identity(), identity());
    let space = space(b"own-doc");
    let (doc, proven) = authored(44, &me, space);
    world.insert_resource(view_of(
        &replicas,
        &me,
        TrustTable::new(DeviceStorage::memory()),
    ));
    let relayer = remote(&mut world, peer(3), Some(bob.did()));
    assert!(spawn_pin(&mut world, &relayer, proven));
    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    load(&mut world, tracker);

    reconcile(&mut world);

    assert!(replicas.pins_as_author(peer(1), doc));
    assert!(world.get::<NoLocalPin>(tracker).is_none());
}

#[test]
fn reconcile_does_not_repin_a_doc_the_local_peer_withdrew() {
    let (mut world, replicas) = world();
    let (me, bob) = (identity(), identity());
    let space = space(b"own-doc-withdrawn");
    join(&replicas, space);
    let (doc, claim) = authored(46, &me, space);
    let (_, second) = authored(46, &me, space);
    world.insert_resource(view_of(
        &replicas,
        &me,
        TrustTable::new(DeviceStorage::memory()),
    ));
    let laptop = local(&mut world, peer(1), me.did());
    assert!(spawn_pin(&mut world, &laptop, claim));
    let relayer = remote(&mut world, peer(3), Some(bob.did()));
    assert!(spawn_pin(&mut world, &relayer, second));
    let key = NamespaceSecret::from_bytes(&[46; 32]);
    withdraw_pin(&mut world, peer(1), doc, &key).expect("withdraw");
    assert!(!replicas.pins_by(peer(1), doc));
    let tracker = index::lookup::<PinnedDoc>(&world, doc).expect("tracker");
    load(&mut world, tracker);

    reconcile(&mut world);

    assert!(
        !replicas.pins_by(peer(1), doc),
        "a deleted document stays unpinned"
    );
}

#[test]
fn reconcile_pins_once_the_author_becomes_known() {
    let (mut world, replicas) = world();
    let (alice, me) = (identity(), identity());
    let space = space(b"late-author");
    let (doc, proven) = authored(45, &alice, space);
    world.insert_resource(view_of(
        &replicas,
        &me,
        TrustTable::new(DeviceStorage::memory()),
    ));
    let tracker = world.spawn(PinnedDoc { doc, space }).id();
    load(&mut world, tracker);

    reconcile(&mut world);
    assert!(!replicas.pins_by(peer(1), doc), "no author is known yet");

    let author = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &author, proven));
    reconcile(&mut world);
    assert!(replicas.pins_by(peer(1), doc));
}

/// Converts the proven pins of a validated snapshot into what the world
/// applies, as the wire does after checking each proof.
fn proven_docs(docs: Vec<ReceivedDoc>, author: &Did) -> Vec<RemoteDoc> {
    docs.into_iter()
        .map(|d| RemoteDoc {
            doc:        d.doc,
            space:      d.space,
            pin:        d.pin.map(|pin| PinClaim {
                proven: Proven {
                    author: author.clone(),
                    proof:  pin.author,
                },
                record: pin.record,
            }),
            hold:       d.hold,
            withdrawal: d.withdrawal,
            layers:     d.layers,
        })
        .collect()
}

/// Takes the next queued message off `feed`, validated, and applies the
/// snapshot it carries to `world` as `from`'s.
fn deliver_snapshot(feed: &StateFeed, world: &mut World, from: Entity, author: &Did) {
    let Ok(msg @ ReplicationMsg::Snapshot { .. }) = feed.rx.try_recv() else {
        panic!("a snapshot is queued");
    };
    let Ok(Received::Snapshot { scope, docs, .. }) = msg.validate(clock::current_micros()) else {
        panic!("the snapshot validates");
    };
    apply_remote(
        world,
        from,
        Remote::Snapshot {
            scope,
            docs: proven_docs(docs, author),
        },
    );
}

#[test]
fn a_scoped_snapshot_leaves_the_senders_other_spaces_intact() {
    let (mut world, replicas) = world();
    let alice = identity();
    let (here, there) = (space(b"scope-here"), space(b"scope-there"));
    let (near, near_claim) = authored(31, &alice, here);
    let (far, far_claim) = authored(32, &alice, there);
    let state = spawn_doc(&mut world, there.doc());
    let device = remote(&mut world, peer(2), Some(alice.did()));
    assert!(spawn_pin(&mut world, &device, near_claim));
    assert!(spawn_pin(&mut world, &device, far_claim));
    spawn_hold(
        &mut world,
        &device,
        HoldClaim {
            doc:   far,
            space: there,
            at:    3,
        },
    );
    Speaker {
        source: &device,
        who:    &alice,
        doc:    there.doc(),
        space:  there,
    }
    .say(&mut world, vec![write(&state_k(), b"v")], 1);

    apply_remote(
        &mut world,
        device.entity,
        Remote::Snapshot {
            scope: Some(here),
            docs:  Vec::new(),
        },
    );

    assert!(!replicas.pins_by(peer(2), near), "the scope is replaced");
    assert!(replicas.pins_by(peer(2), far), "other spaces stay");
    assert_eq!(replicas.holder(there, far), Some(peer(2)));
    assert_eq!(resolved(&state, &state_k()).as_deref(), Some(&b"v"[..]));

    apply_remote(
        &mut world,
        device.entity,
        Remote::Snapshot {
            scope: None,
            docs:  Vec::new(),
        },
    );

    assert!(!replicas.pins_by(peer(2), far), "an unscoped one wipes all");
    assert_eq!(resolved(&state, &state_k()), None);
}

/// Peer A instances a space again and asks peer B, who pinned a document in
/// it, to restate: no presence, leave or gossip is involved.
#[test]
fn a_peers_pin_returns_after_the_space_is_instanced_again() {
    let (mut a_world, a_replicas) = world();
    let (mut b_world, b_replicas) = world();
    a_world.add_observer(request_on_space_added);
    let bob = identity();
    let (a_id, b_id) = (peer(1), peer(2));
    let beacon_space = space(b"beacon");
    let (beacon, claim) = authored(41, &bob, beacon_space);

    let b_source = local(&mut b_world, b_id, bob.did());
    assert!(spawn_pin(&mut b_world, &b_source, claim));
    let b_feed = b_replicas.register_stream(b_id, a_id);
    let a_feed = a_replicas.register_stream(a_id, b_id);
    assert!(a_feed.rx.try_recv().is_ok(), "the opening snapshot");
    a_world.spawn((
        Peer(EndpointAddr::new(b_id)),
        LastSeenIn::heard_in(beacon_space, Duration::ZERO),
    ));
    let b_peer = remote(&mut a_world, b_id, Some(bob.did()));
    let instance = a_world.spawn(Space(beacon_space)).id();
    assert!(
        matches!(a_feed.rx.try_recv(), Ok(ReplicationMsg::Request { .. })),
        "first load"
    );
    deliver_snapshot(&b_feed, &mut a_world, b_peer.entity, bob.did());
    assert!(a_replicas.pins_by(b_id, beacon));

    a_world.despawn(instance);
    assert!(!a_replicas.pins_by(b_id, beacon), "the space took it along");

    a_world.spawn(Space(beacon_space));
    let Ok(request) = a_feed.rx.try_recv() else {
        panic!("instancing the space asks again");
    };
    assert!(a_feed.rx.try_recv().is_err(), "one request");
    let Ok(Received::Request { space }) = request.validate(clock::current_micros()) else {
        panic!("a request validates");
    };
    assert_eq!(space, beacon_space);
    assert!(b_replicas.answer_request(a_id, space, Instant::now()));
    deliver_snapshot(&b_feed, &mut a_world, b_peer.entity, bob.did());

    assert!(a_replicas.pins_by(b_id, beacon), "B's pin is restored");
    a_replicas.unregister_stream(a_feed.token);
    b_replicas.unregister_stream(b_feed.token);
}

#[test]
fn a_tick_whose_root_does_not_match_asks_its_writer_for_the_space() {
    let (mut world, _) = world();
    let guest = identity();
    let space = space(b"root-mismatch");
    let doc = space.doc();
    spawn_doc(&mut world, doc);
    let visitor = remote(&mut world, peer(3), Some(guest.did()));
    let speaker = Speaker {
        source: &visitor,
        who: &guest,
        doc,
        space,
    };
    let replicas = world.resource::<Replicas>().clone();
    let feed = replicas.register_stream(peer(1), peer(3));
    assert!(feed.rx.try_recv().is_ok(), "the opening snapshot");

    let honest = speaker.layer(&[write(&state_k(), b"v")], 1);
    let (writes, clears) = check_tick(&[write(&state_k(), b"v")], &[]).expect("within bounds");
    let tick = ReceivedTick {
        writes,
        clears,
        state: honest.state,
    };
    layers::apply_tick(&mut world, &visitor, &tick);
    assert!(feed.rx.try_recv().is_err(), "a matching root asks nothing");

    speaker.say(&mut world, vec![write(&state_k(), b"w")], 2);
    assert!(
        matches!(feed.rx.try_recv(), Ok(ReplicationMsg::Request { space: asked }) if asked == space),
        "a root that differs asks for the space"
    );
    replicas.unregister_stream(feed.token);
}
