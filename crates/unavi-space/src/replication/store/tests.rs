use hsd::id::PrimId;
use iroh_docs::NamespaceSecret;
use unavi_identity::identity::Identity;
use unavi_local::DeviceStorage;
use unavi_policy::trust::{
    Trust,
    TrustTable,
};
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::{
    holds::MAX_HOLDS_PER_PEER,
    requests::{
        REQUEST_BURST,
        REQUEST_INTERVAL,
    },
    *,
};
use crate::replication::{
    message::{
        HoldClaim,
        Received,
        session::{
            LayerState,
            ReceivedLayer,
            ReceivedTick,
            SignedLayer,
        },
    },
    record::PinRecord,
    session::{
        SessionKey,
        WriterId,
        cells::Cells,
        layers::{
            LocalTick,
            MAX_LAYERS_PER_DOC,
        },
    },
};

/// A writable namespace, its document id, and a proof that `author` wrote it.
fn authored(seed: u8, author: &Identity) -> (DocId, Authorship) {
    let ns = NamespaceSecret::from_bytes(&[seed; 32]);
    let proof = Authorship::sign(&ns, author).expect("sign");
    (DocId(*ns.id().as_bytes()), proof)
}

/// A record saying `doc` is pinned into `space` as of `at`. The store takes
/// records as verified, so any key signs it.
fn pin_record(doc: DocId, space: SpaceId, at: u64) -> Record {
    let key = NamespaceSecret::from_bytes(&[0xAA; 32]);
    Record::sign(
        &key,
        PinRecord {
            doc,
            space,
            at,
            pinned: true,
        },
    )
    .expect("sign")
}

fn identity() -> Identity {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    Identity::new(did, key)
}

fn space(seed: &[u8]) -> SpaceId {
    SpaceId(*blake3::hash(seed).as_bytes())
}

/// A distinct, valid endpoint id per seed. Arbitrary bytes are not a curve
/// point, so a key has to be derived rather than written down.
fn peer(seed: u8) -> EndpointId {
    iroh::SecretKey::from_bytes(&[seed; 32]).public()
}

/// A replica whose local identity has loaded, so other DIDs are judged by the
/// trust table's default.
fn replicas() -> Replicas {
    let replicas = Replicas::new(Policy::new(), Rules::default());
    replicas.attribution().set_local(
        identity().did().clone(),
        TrustTable::new(DeviceStorage::memory()),
    );
    replicas
}

#[test]
fn the_author_endpoint_is_the_authors_first_device_to_pin() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(1, &alice);
    let space = space(b"author-endpoint");
    let (laptop, phone) = (peer(1), peer(2));

    replicas
        .add_pin(
            phone,
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof:  proof.clone(),
            },
            pin_record(doc, space, 20),
        )
        .expect("pin");
    replicas
        .add_pin(
            laptop,
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 30),
        )
        .expect("pin");
    assert_eq!(replicas.author_endpoint(space, doc), Some(phone));
    assert_eq!(replicas.author(doc).as_ref(), Some(alice.did()));

    replicas.remove_pin(phone, doc);
    assert_eq!(replicas.author_endpoint(space, doc), Some(laptop));
    replicas.remove_pin(laptop, doc);
    assert_eq!(replicas.author_endpoint(space, doc), None);
    assert!(!replicas.has_doc(space, doc));
}

#[test]
fn a_second_author_cannot_claim_a_pinned_document() {
    let replicas = replicas();
    let (alice, mallory) = (identity(), identity());
    let (doc, alice_proof) = authored(2, &alice);
    let (_, mallory_proof) = authored(2, &mallory);
    let space = space(b"second-author");

    replicas
        .add_pin(
            peer(1),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof:  alice_proof,
            },
            pin_record(doc, space, 1),
        )
        .expect("pin");
    assert_eq!(
        replicas.add_pin(
            peer(2),
            Some(mallory.did()),
            Proven {
                author: mallory.did().clone(),
                proof:  mallory_proof,
            },
            pin_record(doc, space, 0)
        ),
        Err(PinRefused::WrongAuthor)
    );
    assert_eq!(replicas.author_endpoint(space, doc), Some(peer(1)));
}

/// The first pin decides a document's space, and a later pin cannot move it.
#[test]
fn a_pin_naming_another_space_is_refused() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(3, &alice);
    let (real, bogus) = (space(b"real"), space(b"bogus"));

    replicas
        .add_pin(
            peer(1),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof:  proof.clone(),
            },
            pin_record(doc, real, 1),
        )
        .expect("pin");
    assert_eq!(
        replicas.add_pin(
            peer(2),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, bogus, 2)
        ),
        Err(PinRefused::WrongSpace)
    );
    assert!(replicas.has_doc(real, doc));
}

#[test]
fn a_hold_heard_before_the_pin_counts_once_the_document_is_present() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(4, &alice);
    let space = space(b"hold");
    let (author_device, guest) = (peer(1), peer(2));

    assert!(replicas.add_hold(guest, HoldClaim { doc, space, at: 5 }));
    assert_eq!(
        replicas.holder(space, doc),
        None,
        "nothing holds an absent document"
    );

    replicas
        .add_pin(
            author_device,
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 1),
        )
        .expect("pin");
    assert_eq!(replicas.holder(space, doc), Some(guest));

    replicas.remove_hold(guest, doc);
    assert_eq!(replicas.holder(space, doc), Some(author_device));
}

#[test]
fn a_peer_holds_a_bounded_number_of_documents() {
    let replicas = replicas();
    let space = space(b"hold-cap");
    let doc = |n: usize| {
        let mut bytes = [0; 32];
        bytes[..8].copy_from_slice(&(n as u64).to_le_bytes());
        DocId(bytes)
    };
    for n in 0..MAX_HOLDS_PER_PEER {
        assert!(replicas.add_hold(
            peer(1),
            HoldClaim {
                doc: doc(n),
                space,
                at: 1,
            }
        ));
    }
    assert!(!replicas.add_hold(
        peer(1),
        HoldClaim {
            doc: doc(MAX_HOLDS_PER_PEER),
            space,
            at: 1,
        }
    ));
    assert!(
        replicas.add_hold(
            peer(1),
            HoldClaim {
                doc: doc(0),
                space,
                at: 2,
            }
        ),
        "restating a held document is not a new hold"
    );
    replicas.remove_hold(peer(1), doc(0));
    assert!(replicas.add_hold(
        peer(1),
        HoldClaim {
            doc: doc(MAX_HOLDS_PER_PEER),
            space,
            at: 1,
        }
    ));
}

#[test]
fn a_non_author_pin_keeps_presence_but_never_resolves_the_author_endpoint() {
    let replicas = replicas();
    let (alice, bob) = (identity(), identity());
    let (doc, proof) = authored(9, &alice);
    let space = space(b"non-author-pin");
    let (relayer, anonymous, author_device) = (peer(1), peer(2), peer(3));

    replicas
        .add_pin(
            relayer,
            Some(bob.did()),
            Proven {
                author: alice.did().clone(),
                proof:  proof.clone(),
            },
            pin_record(doc, space, 1),
        )
        .expect("pin");
    replicas
        .add_pin(
            anonymous,
            None,
            Proven {
                author: alice.did().clone(),
                proof:  proof.clone(),
            },
            pin_record(doc, space, 2),
        )
        .expect("pin");
    assert!(replicas.is_pinned(doc));
    assert!(!replicas.is_author_pinned(doc));
    assert!(!replicas.pins_as_author(relayer, doc));
    assert_eq!(replicas.author_endpoint(space, doc), None);
    assert_eq!(replicas.holder(space, doc), None);
    assert_eq!(
        replicas.proven(doc).map(|p| p.author),
        Some(alice.did().clone())
    );

    replicas
        .add_pin(
            author_device,
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 9),
        )
        .expect("pin");
    assert!(replicas.pins_as_author(author_device, doc));
    assert_eq!(
        replicas.author_endpoint(space, doc),
        Some(author_device),
        "an earlier non-author pin does not outrank the author"
    );
    assert_eq!(replicas.holder(space, doc), Some(author_device));

    replicas.remove_pin(author_device, doc);
    assert!(!replicas.is_author_pinned(doc));
    assert!(replicas.is_pinned(doc), "the other pins keep it present");
    assert_eq!(replicas.author_endpoint(space, doc), None);
    assert_eq!(replicas.holder(space, doc), None);
    assert!(replicas.has_doc(space, doc));
}

#[test]
fn sync_sources_include_non_author_pinners() {
    let replicas = replicas();
    let (alice, bob) = (identity(), identity());
    let (doc, proof) = authored(10, &alice);
    let space = space(b"sync-sources");
    let (relayer, me) = (peer(1), peer(2));

    replicas
        .add_pin(
            relayer,
            Some(bob.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 1),
        )
        .expect("pin");
    assert_eq!(replicas.sync_sources(doc, me), vec![relayer]);
}

fn doc_of(seed: u8) -> DocId {
    DocId(*NamespaceSecret::from_bytes(&[seed; 32]).id().as_bytes())
}

/// The record the namespace of `doc_of(seed)` signs.
fn record_for(seed: u8, space: SpaceId, at: u64, pinned: bool) -> Record {
    let payload = PinRecord {
        doc: doc_of(seed),
        space,
        at,
        pinned,
    };
    Record::sign(&NamespaceSecret::from_bytes(&[seed; 32]), payload).expect("sign")
}

/// Puts the local peer in `space`, as a loaded space entity does.
fn join(replicas: &Replicas, space: SpaceId) {
    replicas
        .attribution()
        .policy()
        .update(space.doc(), |record| record.space = Some(space.doc()));
}

#[test]
fn the_newest_record_wins_and_a_withdrawal_wins_a_tie() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(11, &alice);
    let space = space(b"records");
    replicas
        .add_pin(
            peer(1),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 1),
        )
        .expect("pin");

    replicas.observe_record(&record_for(11, space, 5, true), peer(1));
    replicas.observe_record(&record_for(11, space, 3, true), peer(1));
    assert_eq!(
        replicas.newest_record_at(doc),
        Some(5),
        "an older pin record is ignored"
    );
    assert!(replicas.is_pin_current(doc));

    replicas.observe_record(&record_for(11, space, 5, false), peer(1));
    assert!(replicas.is_withdrawn(doc), "a withdrawal wins a tie");
    replicas.observe_record(&record_for(11, space, 4, true), peer(1));
    assert!(
        replicas.is_withdrawn(doc),
        "a pin older than the withdrawal does not undo it"
    );

    replicas.observe_record(&record_for(11, space, 6, true), peer(1));
    assert!(!replicas.is_withdrawn(doc));
    assert_eq!(replicas.pinned_record(doc).map(|r| r.stamp), Some(6));
}

#[test]
fn a_record_for_another_space_than_the_documents_is_ignored() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(12, &alice);
    let (real, other) = (space(b"records-real"), space(b"records-other"));
    replicas
        .add_pin(
            peer(1),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, real, 1),
        )
        .expect("pin");

    replicas.observe_record(&record_for(12, other, 9, true), peer(1));
    replicas.observe_record(&record_for(12, other, 9, false), peer(1));

    assert!(
        replicas.is_pin_current(doc),
        "the pin's own record stands over another space's newer withdrawal"
    );
    assert!(!replicas.is_withdrawn(doc));
}

#[test]
fn only_records_for_spaces_the_local_peer_is_in_are_kept() {
    let replicas = replicas();
    let space = space(b"records-gated");

    replicas.observe_record(&record_for(14, space, 2, false), peer(1));
    assert!(
        !replicas.is_withdrawn(doc_of(14)),
        "the local peer is not in the space"
    );

    join(&replicas, space);
    replicas.observe_record(&record_for(14, space, 2, false), peer(1));
    assert!(replicas.is_withdrawn(doc_of(14)));
}

#[test]
fn hearsay_is_budgeted_per_source_and_goes_when_its_source_does() {
    let replicas = replicas();
    let space = space(b"hearsay");
    join(&replicas, space);
    let (flooder, honest) = (peer(1), peer(2));

    replicas.observe_record(&record_for(1, space, 2, false), honest);
    for seed in 10..=u8::MAX {
        replicas.observe_record(&record_for(seed, space, 2, false), flooder);
    }
    let heard = replicas.heard_from(flooder);
    assert_eq!(heard, 64, "a source holds at most its budget");
    assert!(
        replicas.is_withdrawn(doc_of(1)),
        "a flood does not touch another source's"
    );
    assert_eq!(replicas.known_in(space), 0, "hearsay is not known");

    replicas.drop_hearsay(flooder);
    assert_eq!(replicas.heard_from(flooder), 0);
    assert!(!replicas.is_withdrawn(doc_of(u8::MAX)));
    assert!(replicas.is_withdrawn(doc_of(1)));
}

#[test]
fn hearsay_becomes_known_when_the_document_gains_presence() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(15, &alice);
    let space = space(b"hearsay-promoted");
    join(&replicas, space);

    replicas.observe_record(&record_for(15, space, 5, false), peer(2));
    replicas
        .add_pin(
            peer(3),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 1),
        )
        .expect("a stale pin");
    replicas.observe_record(&record_for(15, space, 1, true), peer(3));
    assert!(
        replicas.is_withdrawn(doc),
        "the withdrawal heard first outranks the stale pin"
    );

    replicas.drop_hearsay(peer(2));
    assert!(
        replicas.is_withdrawn(doc),
        "it is known now, not merely heard"
    );
    assert_eq!(replicas.known_in(space), 1);
    assert_eq!(replicas.heard_from(peer(2)), 0);
}

#[test]
fn a_known_record_outlives_its_presence_and_a_recreated_one_cannot_regress() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(16, &alice);
    let space = space(b"known-retained");
    join(&replicas, space);
    let pin = |at| {
        replicas
            .add_pin(
                peer(1),
                Some(alice.did()),
                Proven {
                    author: alice.did().clone(),
                    proof:  proof.clone(),
                },
                pin_record(doc, space, at),
            )
            .expect("pin");
    };

    pin(1);
    replicas.observe_record(&record_for(16, space, 5, true), peer(1));
    replicas.remove_pin(peer(1), doc);
    assert!(!replicas.has_doc(space, doc), "the presence is gone");
    assert_eq!(
        replicas.newest_record_at(doc),
        Some(5),
        "the record is kept"
    );

    pin(2);
    replicas.observe_record(&record_for(16, space, 3, true), peer(1));
    assert_eq!(
        replicas.newest_record_at(doc),
        Some(5),
        "an older record does not replace it"
    );
}

#[test]
fn a_record_is_not_kept_past_the_presence_of_a_space_the_peer_is_not_in() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(17, &alice);
    let space = space(b"known-dropped");

    replicas
        .add_pin(
            peer(1),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 1),
        )
        .expect("pin");
    replicas.observe_record(&record_for(17, space, 5, true), peer(1));
    assert_eq!(
        replicas.newest_record_at(doc),
        Some(5),
        "a present document keeps its record"
    );

    replicas.remove_pin(peer(1), doc);
    assert_eq!(replicas.newest_record_at(doc), None);
}

#[test]
fn a_flood_of_hearsay_does_not_evict_a_known_withdrawal() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(18, &alice);
    let space = space(b"flood");
    join(&replicas, space);
    replicas
        .add_pin(
            peer(1),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 1),
        )
        .expect("pin");
    replicas.observe_record(&record_for(18, space, 5, false), peer(1));
    replicas.remove_pin(peer(1), doc);

    for seed in 20..=u8::MAX {
        replicas.observe_record(&record_for(seed, space, 2, false), peer(9));
    }

    assert!(replicas.is_withdrawn(doc));
    assert_eq!(replicas.known_in(space), 1);
}

#[test]
fn a_withdrawal_is_listed_in_snapshots_only_while_the_peer_is_in_its_space() {
    let replicas = replicas();
    let space = space(b"withdrawal-listed");
    join(&replicas, space);
    replicas.observe_record(&record_for(13, space, 2, false), peer(2));

    let listed = |replicas: &Replicas| {
        let feed = replicas.register_stream(peer(1), peer(9));
        let Ok(ReplicationMsg::Snapshot { docs, .. }) = feed.rx.try_recv() else {
            panic!("a stream opens with a snapshot");
        };
        replicas.unregister_stream(feed.token);
        docs.into_iter().find(|d| d.doc == doc_of(13))
    };
    let doc = listed(&replicas).expect("a withdrawn doc with no presence is still listed");
    assert!(doc.pin.is_none() && doc.hold.is_none());
    assert!(doc.withdrawal.is_some());

    replicas
        .attribution()
        .policy()
        .update(space.doc(), |record| record.space = None);
    assert!(
        listed(&replicas).is_none(),
        "a space the peer has left is not told"
    );

    join(&replicas, space);
    replicas.forget_space_records(space);
    assert!(listed(&replicas).is_none());
    assert!(!replicas.is_withdrawn(doc_of(13)));
}

#[test]
fn a_pin_snapshot_carries_the_pins_own_record_and_the_withdrawal_apart() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(19, &alice);
    let space = space(b"snapshot-fields");
    join(&replicas, space);
    let me = peer(1);

    replicas
        .add_pin(
            me,
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 1),
        )
        .expect("pin");
    replicas.observe_record(&record_for(19, space, 9, false), peer(2));

    let feed = replicas.register_stream(me, peer(9));
    let Ok(ReplicationMsg::Snapshot { docs, .. }) = feed.rx.try_recv() else {
        panic!("a stream opens with a snapshot");
    };
    replicas.unregister_stream(feed.token);
    let listed = docs.iter().find(|d| d.doc == doc).expect("listed");
    let pin = listed
        .pin
        .as_ref()
        .expect("the sender's pin is stated whatever the newest record says");
    assert!(pin.record.payload().expect("decode").pinned);
    assert!(
        !listed
            .withdrawal
            .as_ref()
            .expect("withdrawal")
            .payload()
            .expect("decode")
            .pinned
    );
}
/// One writer's layer on one document, for building its ticks and states.
struct Writer<'a> {
    device: EndpointId,
    who:    &'a Identity,
    doc:    DocId,
    space:  SpaceId,
}

impl Writer<'_> {
    fn id(&self) -> WriterId {
        WriterId::new(self.device, self.who.did())
    }

    fn state(&self, at: u64, root: [u8; 32]) -> SignedLayer {
        SignedLayer::sign(
            LayerState {
                device: self.device,
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

    /// A tick at `at` setting `writes` and clearing `clears`. A live tick's
    /// root is not checked, so it is left zero.
    fn tick(&self, at: u64, writes: &[(&str, &[u8])], clears: &[&str]) -> ReceivedTick {
        ReceivedTick {
            writes: writes
                .iter()
                .map(|(field, value)| (key(field), Some(value.to_vec())))
                .collect(),
            clears: clears.iter().map(|field| key(field)).collect(),
            state:  self.state(at, [0; 32]),
        }
    }

    /// A whole layer at `at` holding `cells`, signed over their root.
    fn layer(&self, at: u64, cells: &[(&str, &[u8])]) -> ReceivedLayer {
        let mut held = Cells::default();
        for (field, value) in cells {
            held.set(key(field), Some(value.to_vec()));
        }
        ReceivedLayer {
            state: self.state(at, held.root()),
            cells: held,
        }
    }
}

fn key(field: &str) -> SessionKey {
    SessionKey {
        prim: PrimId([1; 16]),
        name: format!("state/{field}").parse().expect("name"),
    }
}

/// `writer`'s cells on `doc`, as `(field, value)`.
fn cells_of(replicas: &Replicas, writer: WriterId, doc: DocId) -> Vec<(String, Vec<u8>)> {
    let mut cells = replicas
        .session_opinions(writer, doc)
        .into_iter()
        .map(|(key, value, _)| {
            let field = key.name.to_string().trim_start_matches("state/").to_owned();
            (field, value.unwrap_or_default())
        })
        .collect::<Vec<_>>();
    cells.sort();
    cells
}

fn cell(field: &str, value: &[u8]) -> (String, Vec<u8>) {
    (field.to_owned(), value.to_vec())
}

#[test]
fn a_tick_lands_whole_in_its_writers_own_layer() {
    let replicas = replicas();
    let space = space(b"ticks");
    let (alice, bob) = (identity(), identity());
    let a = Writer {
        device: peer(1),
        who: &alice,
        doc: space.doc(),
        space,
    };
    let b = Writer {
        device: peer(2),
        who: &bob,
        ..a
    };

    replicas
        .apply_tick(a.device, &a.tick(1, &[("k", b"a"), ("gone", b"x")], &[]))
        .expect("tick");
    replicas
        .apply_tick(b.device, &b.tick(1, &[("k", b"b")], &[]))
        .expect("tick");
    let applied = replicas
        .apply_tick(a.device, &a.tick(2, &[("gone", b"y")], &["gone", "k"]))
        .expect("tick");

    assert_eq!(
        cells_of(&replicas, a.id(), a.doc),
        vec![cell("gone", b"y")],
        "a key the tick both clears and writes keeps the write"
    );
    assert_eq!(applied.cleared, vec![key("k")]);
    assert_eq!(cells_of(&replicas, b.id(), b.doc), vec![cell("k", b"b")]);
}

#[test]
fn a_tick_no_newer_than_its_layer_is_refused() {
    let replicas = replicas();
    let space = space(b"stale");
    let alice = identity();
    let a = Writer {
        device: peer(1),
        who: &alice,
        doc: space.doc(),
        space,
    };
    replicas
        .apply_tick(a.device, &a.tick(5, &[("k", b"new")], &[]))
        .expect("tick");
    assert_eq!(
        replicas
            .apply_tick(a.device, &a.tick(5, &[("k", b"old")], &[]))
            .err(),
        Some(SessionError::Stale)
    );
    assert_eq!(cells_of(&replicas, a.id(), a.doc), vec![cell("k", b"new")]);
}

#[test]
fn a_tick_refused_by_quota_leaves_nothing_and_holds_back_what_follows() {
    let replicas = replicas();
    let space = space(b"atomic");
    let alice = identity();
    let a = Writer {
        device: peer(1),
        who: &alice,
        doc: space.doc(),
        space,
    };
    let limit = Rules::default().limits(Trust::Guest).stock[&Stock::SessionMemory];
    let huge = vec![0; usize::try_from(limit).expect("fits") + 1];

    replicas
        .apply_tick(a.device, &a.tick(1, &[("small", b"v")], &[]))
        .expect("tick");
    assert_eq!(
        replicas
            .apply_tick(a.device, &a.tick(2, &[("big", &huge)], &[]))
            .err(),
        Some(SessionError::QuotaExceeded)
    );
    assert!(replicas.session_writers(a.doc).is_empty(), "the layer goes");
    let quota = replicas
        .attribution()
        .writer_quota(a.device, Some(alice.did()));
    assert_eq!(quota.usage(Stock::Documents), 0, "no layer charge");
    assert_eq!(quota.usage(Stock::SessionMemory), 0, "no opinion charge");

    assert_eq!(
        replicas
            .apply_tick(a.device, &a.tick(3, &[("small", b"w")], &[]))
            .err(),
        Some(SessionError::QuotaExceeded),
        "a later tick builds on the refused one"
    );
    replicas
        .replace_layer(a.device, a.layer(4, &[("small", b"w")]))
        .expect("a whole layer");
    replicas
        .apply_tick(a.device, &a.tick(5, &[("more", b"x")], &[]))
        .expect("ticks follow a whole layer");
    assert_eq!(
        cells_of(&replicas, a.id(), a.doc),
        vec![cell("more", b"x"), cell("small", b"w")]
    );
}

#[test]
fn a_layer_naming_another_space_than_the_documents_is_refused() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(11, &alice);
    let (real, other) = (space(b"session-real"), space(b"session-other"));
    replicas
        .add_pin(
            peer(1),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, real, 1),
        )
        .expect("pin");
    let a = Writer {
        device: peer(2),
        who: &alice,
        doc,
        space: other,
    };
    assert_eq!(
        replicas
            .apply_tick(a.device, &a.tick(1, &[("k", b"v")], &[]))
            .err(),
        Some(SessionError::WrongSpace)
    );
}

#[test]
fn a_document_holds_no_more_layers_than_the_cap_but_its_author_may_exceed_it() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(12, &alice);
    let space = space(b"cap");
    replicas
        .add_pin(
            peer(200),
            Some(alice.did()),
            Proven {
                author: alice.did().clone(),
                proof,
            },
            pin_record(doc, space, 1),
        )
        .expect("pin");

    let strangers = (0..=MAX_LAYERS_PER_DOC)
        .map(|_| identity())
        .collect::<Vec<_>>();
    let writer = |n: usize| Writer {
        device: peer(u8::try_from(n + 1).expect("seed")),
        who: &strangers[n],
        doc,
        space,
    };
    for n in 0..MAX_LAYERS_PER_DOC {
        let w = writer(n);
        replicas
            .apply_tick(w.device, &w.tick(1, &[("k", b"v")], &[]))
            .expect("under the cap");
    }
    let past = writer(MAX_LAYERS_PER_DOC);
    assert_eq!(
        replicas
            .apply_tick(past.device, &past.tick(1, &[("k", b"v")], &[]))
            .err(),
        Some(SessionError::QuotaExceeded)
    );
    let author = Writer {
        device: peer(200),
        who: &alice,
        doc,
        space,
    };
    replicas
        .apply_tick(author.device, &author.tick(1, &[("k", b"v")], &[]))
        .expect("the author takes a reserved layer");
}

#[test]
fn a_forged_device_claim_is_a_layer_of_its_own() {
    let replicas = replicas();
    let space = space(b"squat");
    let (alice, mallory) = (identity(), identity());
    let real = Writer {
        device: peer(1),
        who: &alice,
        doc: space.doc(),
        space,
    };
    let forged = Writer {
        who: &mallory,
        ..real
    };

    replicas
        .replace_layer(peer(3), forged.layer(9, &[("k", b"forged")]))
        .expect("a layer of mallory's");
    replicas
        .replace_layer(peer(4), real.layer(1, &[("k", b"real")]))
        .expect("alice's layer is not displaced");
    assert_eq!(
        cells_of(&replicas, real.id(), real.doc),
        vec![cell("k", b"real")]
    );
    assert_eq!(
        cells_of(&replicas, forged.id(), forged.doc),
        vec![cell("k", b"forged")]
    );
}

#[test]
fn a_whole_layer_replaces_only_an_older_one_and_reports_what_it_dropped() {
    let replicas = replicas();
    let space = space(b"replace");
    let alice = identity();
    let a = Writer {
        device: peer(1),
        who: &alice,
        doc: space.doc(),
        space,
    };
    replicas
        .replace_layer(peer(2), a.layer(2, &[("kept", b"1"), ("dropped", b"1")]))
        .expect("layer");

    let applied = replicas
        .replace_layer(peer(3), a.layer(3, &[("kept", b"1"), ("new", b"1")]))
        .expect("newer");
    assert_eq!(applied.cleared, vec![key("dropped")]);
    assert_eq!(
        applied
            .set
            .iter()
            .map(|(key, ..)| key.clone())
            .collect::<Vec<_>>(),
        vec![key("new")],
        "an unchanged cell is not composed again"
    );

    replicas
        .replace_layer(peer(4), a.layer(2, &[("stale", b"1")]))
        .expect("older is a no-op");
    assert_eq!(
        cells_of(&replicas, a.id(), a.doc),
        vec![cell("kept", b"1"), cell("new", b"1")]
    );
    let mut sources = replicas.session_sources(a.id(), a.doc);
    sources.sort();
    let mut expected = vec![peer(2), peer(3)];
    expected.sort();
    assert_eq!(
        sources, expected,
        "a holder of an older state is not a source of this one"
    );
}

#[test]
fn a_layer_goes_only_when_its_last_source_does() {
    let replicas = replicas();
    let space = space(b"sources");
    let alice = identity();
    let a = Writer {
        device: peer(1),
        who: &alice,
        doc: space.doc(),
        space,
    };
    replicas
        .apply_tick(a.device, &a.tick(1, &[("k", b"v")], &[]))
        .expect("tick");
    assert!(replicas.add_session_source(a.id(), a.doc, peer(2)));
    assert!(!replicas.remove_session_source(a.id(), a.doc, a.device));
    assert_eq!(cells_of(&replicas, a.id(), a.doc), vec![cell("k", b"v")]);
    assert!(replicas.remove_session_source(a.id(), a.doc, peer(2)));
    assert_eq!(replicas.session_writers(a.doc), Vec::new());
}

#[test]
fn a_local_write_signs_the_root_of_the_cells_it_leaves() {
    let replicas = replicas();
    let space = space(b"local-root");
    let alice = identity();
    let (writes, clears) = (vec![(key("k"), Some(b"v".to_vec()))], Vec::new());
    let tick = LocalTick {
        device: peer(1),
        did: alice.did(),
        doc: space.doc(),
        space,
        writes: &writes,
        clears: &clears,
    };
    let (_, state) = replicas
        .write_local(&tick, |payload| {
            SignedLayer::sign(payload, alice.signing_key())
        })
        .expect("write");

    let feed = replicas.register_stream(peer(1), peer(9));
    let Ok(msg @ ReplicationMsg::Snapshot { .. }) = feed.rx.try_recv() else {
        panic!("a stream opens with a snapshot");
    };
    replicas.unregister_stream(feed.token);
    let Ok(Received::Snapshot { mut docs, .. }) = msg.validate(0) else {
        panic!("the snapshot validates");
    };
    let mut layer = docs.remove(0).layers.remove(0);
    assert_eq!(layer.state.payload.root, state.payload.root);
    assert!(
        layer.matches_root(),
        "a relayed copy verifies against the signed root"
    );
}

#[test]
fn a_request_queues_a_snapshot_of_only_its_space() {
    let replicas = replicas();
    let alice = identity();
    let (doc_a, proof_a) = authored(21, &alice);
    let (doc_b, proof_b) = authored(22, &alice);
    let (space_a, space_b) = (space(b"request-a"), space(b"request-b"));
    join(&replicas, space_a);
    join(&replicas, space_b);
    let me = peer(1);

    let toward = replicas.register_stream(me, peer(8));
    let elsewhere = replicas.register_stream(me, peer(9));
    for feed in [&toward, &elsewhere] {
        assert!(matches!(
            feed.rx.try_recv(),
            Ok(ReplicationMsg::Snapshot { .. })
        ));
    }
    for (doc, proof, space) in [(doc_a, proof_a, space_a), (doc_b, proof_b, space_b)] {
        replicas
            .add_pin(
                me,
                Some(alice.did()),
                Proven {
                    author: alice.did().clone(),
                    proof,
                },
                pin_record(doc, space, 1),
            )
            .expect("pin");
    }

    assert!(replicas.answer_request(peer(8), space_a, Instant::now()));

    let Ok(ReplicationMsg::Snapshot { scope, docs, .. }) = toward.rx.try_recv() else {
        panic!("the request is answered with a snapshot");
    };
    assert_eq!(scope, Some(space_a));
    assert_eq!(1, docs.len(), "only the requested space");
    assert!(docs[0].doc == doc_a && docs[0].pin.is_some());
    assert!(toward.rx.try_recv().is_err(), "nothing else is queued");
    assert!(
        elsewhere.rx.try_recv().is_err(),
        "another peer is not answered"
    );
    replicas.unregister_stream(toward.token);
    replicas.unregister_stream(elsewhere.token);
}

#[test]
fn a_request_for_a_space_held_nothing_in_is_answered_empty() {
    let replicas = replicas();
    let toward = replicas.register_stream(peer(1), peer(8));
    assert!(toward.rx.try_recv().is_ok());
    let unknown = space(b"request-unknown");

    assert!(replicas.answer_request(peer(8), unknown, Instant::now()));

    let Ok(ReplicationMsg::Snapshot { scope, docs, .. }) = toward.rx.try_recv() else {
        panic!("an empty snapshot answers");
    };
    assert_eq!(scope, Some(unknown));
    assert!(docs.is_empty());
    replicas.unregister_stream(toward.token);
}

#[test]
fn requests_past_the_rate_limit_are_dropped() {
    let replicas = replicas();
    let toward = replicas.register_stream(peer(1), peer(8));
    assert!(toward.rx.try_recv().is_ok());
    let (a, b) = (space(b"rate-a"), space(b"rate-b"));
    let start = Instant::now();

    for _ in 0..REQUEST_BURST {
        assert!(replicas.answer_request(peer(8), a, start));
    }
    assert!(!replicas.answer_request(peer(8), a, start), "burst spent");
    assert!(
        replicas.answer_request(peer(8), b, start),
        "another space has its own allowance"
    );
    assert!(
        replicas.answer_request(peer(7), a, start),
        "another peer has its own allowance"
    );
    let half = start + REQUEST_INTERVAL / 2;
    assert!(!replicas.answer_request(peer(8), a, half), "not yet earned");
    let later = start + REQUEST_INTERVAL;
    assert!(
        replicas.answer_request(peer(8), a, later),
        "one earned back"
    );
    assert!(!replicas.answer_request(peer(8), a, later), "only one");

    let mut answered = 0;
    while toward.rx.try_recv().is_ok() {
        answered += 1;
    }
    assert_eq!(
        REQUEST_BURST + 1 + 1,
        answered,
        "dropped requests queue nothing"
    );
    replicas.unregister_stream(toward.token);
}

#[test]
fn a_request_goes_out_over_the_stream_to_its_peer() {
    let replicas = replicas();
    let toward = replicas.register_stream(peer(1), peer(8));
    assert!(toward.rx.try_recv().is_ok());
    let s = space(b"request-out");

    assert!(!replicas.request(peer(7), s), "no stream to that peer");
    assert!(replicas.request(peer(8), s));

    let Ok(ReplicationMsg::Request { space }) = toward.rx.try_recv() else {
        panic!("the request is queued");
    };
    assert_eq!(space, s);
    replicas.unregister_stream(toward.token);
}
