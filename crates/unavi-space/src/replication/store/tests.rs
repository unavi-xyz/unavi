use iroh_docs::NamespaceSecret;
use unavi_identity::identity::Identity;
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;

/// A writable namespace, its document id, and a proof that `author` wrote it.
fn authored(seed: u8, author: &Identity) -> (DocId, Authorship) {
    let ns = NamespaceSecret::from_bytes(&[seed; 32]);
    let proof = Authorship::sign(&ns, author).expect("sign");
    (DocId(*ns.id().as_bytes()), proof)
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

fn prim() -> PrimId {
    PrimId([1; 16])
}

fn name(field: &str) -> PropName {
    format!("test/{field}").parse().expect("valid prop name")
}

fn write(field: &str, value: &[u8]) -> Vec<(SessionKey, Option<Vec<u8>>)> {
    vec![(
        SessionKey {
            prim: prim(),
            name: name(field),
        },
        Some(value.to_vec()),
    )]
}

fn replicas() -> Replicas {
    Replicas::new(Policy::new())
}

#[test]
fn the_author_endpoint_is_the_authors_first_device_to_pin() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(1, &alice);
    let space = space(b"author-endpoint");
    let (laptop, phone) = (peer(1), peer(2));

    replicas
        .add_pin(phone, doc, space, alice.did().clone(), proof.clone(), 20)
        .expect("pin");
    replicas
        .add_pin(laptop, doc, space, alice.did().clone(), proof, 30)
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
        .add_pin(peer(1), doc, space, alice.did().clone(), alice_proof, 1)
        .expect("pin");
    assert_eq!(
        replicas.add_pin(peer(2), doc, space, mallory.did().clone(), mallory_proof, 0),
        Err(PinRefused::WrongAuthor)
    );
    assert_eq!(replicas.author_endpoint(space, doc), Some(peer(1)));
}

/// The first pin decides a document's space only because only the author
/// pins; even then a later pin cannot move it.
#[test]
fn a_pin_naming_another_space_is_refused() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(3, &alice);
    let (real, bogus) = (space(b"real"), space(b"bogus"));

    replicas
        .add_pin(peer(1), doc, real, alice.did().clone(), proof.clone(), 1)
        .expect("pin");
    assert_eq!(
        replicas.add_pin(peer(2), doc, bogus, alice.did().clone(), proof, 2),
        Err(PinRefused::WrongSpace)
    );
    assert!(replicas.has_doc(real, doc));
}

#[test]
fn only_the_author_or_whom_the_holder_released_to_may_hold() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(4, &alice);
    let space = space(b"hold");
    let (author_device, guest, other) = (peer(1), peer(2), peer(3));
    let guest_did: Did = "did:web:guest.example".parse().expect("did");

    replicas
        .add_pin(author_device, doc, space, alice.did().clone(), proof, 1)
        .expect("pin");
    assert_eq!(replicas.holder(space, doc), Some(author_device));

    assert!(
        !replicas.add_hold(guest, Some(&guest_did), doc, space, 2),
        "a stranger cannot take physics authority"
    );

    replicas.remove_hold(author_device, doc, Some(guest));
    assert!(!replicas.add_hold(other, None, doc, space, 3));
    assert!(replicas.add_hold(guest, Some(&guest_did), doc, space, 4));
    assert_eq!(replicas.holder(space, doc), Some(guest));

    replicas.remove_hold(guest, doc, None);
    assert_eq!(replicas.holder(space, doc), Some(author_device));
    assert!(
        !replicas.add_hold(guest, Some(&guest_did), doc, space, 5),
        "a release spends the grant"
    );
}

#[test]
fn an_authored_documents_session_is_the_authors_alone() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(5, &alice);
    let space = space(b"perm");
    let stranger: Did = "did:web:stranger.example".parse().expect("did");

    replicas
        .add_pin(peer(1), doc, space, alice.did().clone(), proof, 1)
        .expect("pin");
    assert_eq!(
        replicas.add_session(peer(1), Some(alice.did()), doc, space, write("k", b"v"), 2),
        Ok(())
    );
    assert_eq!(
        replicas.add_session(peer(2), Some(&stranger), doc, space, write("k", b"x"), 3),
        Err(SessionError::NotAuthor)
    );
    assert_eq!(
        replicas.add_session(
            peer(1),
            Some(alice.did()),
            doc,
            SpaceId([0; 32]),
            write("k", b"x"),
            4
        ),
        Err(SessionError::NotAuthor),
        "a document present in one space is not writable through another"
    );
    assert_eq!(
        replicas
            .session_value(space, doc, prim(), &name("k"))
            .as_deref(),
        Some(&b"v"[..])
    );
}

#[test]
fn a_refused_batch_leaves_nothing_behind() {
    let replicas = replicas();
    let space = space(b"atomic");
    let doc = space.doc();
    let mut batch = write("small", b"v");
    batch.push((
        SessionKey {
            prim: prim(),
            name: name("huge"),
        },
        Some(vec![0; 9 * 1024 * 1024]),
    ));

    assert_eq!(
        replicas.add_session(peer(1), None, doc, space, batch, 1),
        Err(SessionError::QuotaExceeded)
    );
    assert_eq!(
        replicas.session_value(space, doc, prim(), &name("small")),
        None
    );
    assert!(!replicas.has_doc(space, doc));
}

#[test]
fn removing_the_last_cell_releases_the_presence() {
    let replicas = replicas();
    let space = space(b"session");
    let doc = space.doc();

    replicas
        .add_session(peer(2), None, doc, space, write("link", b"dest"), 1)
        .expect("anyone writes a space's own document");

    let missing = SessionKey {
        prim: prim(),
        name: name("missing"),
    };
    replicas.remove_session(doc, &missing);
    assert!(replicas.has_doc(space, doc));

    let key = SessionKey {
        prim: prim(),
        name: name("link"),
    };
    replicas.remove_session(doc, &key);
    assert_eq!(
        replicas.session_value(space, doc, prim(), &name("link")),
        None
    );
    assert!(!replicas.has_doc(space, doc));
}

#[test]
fn reverting_a_peer_restores_the_value_it_overwrote() {
    let replicas = replicas();
    let space = space(b"revert");
    let doc = space.doc();
    let (alice, mallory) = (peer(2), peer(3));

    replicas
        .add_session(alice, None, doc, space, write("sign", b"welcome"), 1)
        .expect("alice writes the sign");
    replicas
        .add_session(mallory, None, doc, space, write("sign", b"defaced"), 2)
        .expect("mallory defaces it");

    assert_eq!(replicas.revert_writes(mallory).len(), 1);
    assert_eq!(
        replicas
            .session_value(space, doc, prim(), &name("sign"))
            .as_deref(),
        Some(&b"welcome"[..]),
        "blocking must put back what the blocked peer wrote over"
    );
}

#[test]
fn reverting_drops_a_cell_the_peer_created() {
    let replicas = replicas();
    let space = space(b"revert-new");
    let doc = space.doc();
    let mallory = peer(3);

    replicas
        .add_session(mallory, None, doc, space, write("spam", b"x"), 1)
        .expect("mallory writes a new cell");

    assert_eq!(replicas.revert_writes(mallory).len(), 1);
    assert_eq!(
        replicas.session_value(space, doc, prim(), &name("spam")),
        None
    );
    assert!(
        !replicas.has_doc(space, doc),
        "dropping the last cell must release the document presence"
    );
}

#[test]
fn a_peer_cannot_leave_its_own_earlier_write_as_the_fallback() {
    let replicas = replicas();
    let space = space(b"revert-own");
    let doc = space.doc();
    let mallory = peer(3);

    replicas
        .add_session(mallory, None, doc, space, write("sign", b"first"), 1)
        .expect("first");
    replicas
        .add_session(mallory, None, doc, space, write("sign", b"second"), 2)
        .expect("second");

    assert_eq!(replicas.revert_writes(mallory).len(), 1);
    assert_eq!(
        replicas.session_value(space, doc, prim(), &name("sign")),
        None
    );
}

#[test]
fn a_pin_refused_by_quota_leaves_no_quota_behind() {
    let policy = Policy::new();
    let replicas = Replicas::new(policy.clone());
    let alice = identity();
    let space = space(b"refused");

    // Exhaust the author's document budget.
    let quota = replicas.attribution().author_quota(alice.did());
    while quota.charge(Stock::Documents, 1).is_ok() {}

    let (doc, proof) = authored(9, &alice);
    assert_eq!(
        replicas.add_pin(peer(1), doc, space, alice.did().clone(), proof, 1),
        Err(PinRefused::Quota)
    );
    // A principal kept from the refusal would still roll up into the
    // exhausted author and refuse this charge too.
    assert!(
        policy
            .document_quota(doc, || None)
            .charge(Stock::Documents, 1)
            .is_ok(),
        "a refused pin must not keep a document principal alive"
    );
}

#[test]
fn an_overflowing_stream_resyncs_from_a_snapshot() {
    let replicas = replicas();
    let me = peer(1);
    let feed = replicas.register_stream(me);
    assert!(matches!(
        feed.rx.try_recv(),
        Ok(ReplicationMsg::Snapshot(_))
    ));

    for _ in 0..=OUTBOUND_QUEUE {
        replicas.broadcast(&ReplicationMsg::Unpin {
            doc: DocId([0; 32]),
        });
    }
    assert!(feed.resync.load(Ordering::Relaxed));

    assert!(matches!(
        replicas.resync(&feed, me),
        ReplicationMsg::Snapshot(_)
    ));
    assert!(feed.rx.try_recv().is_err(), "the stale queue is dropped");
    assert!(!feed.resync.load(Ordering::Relaxed));
    replicas.unregister_stream(feed.token);
}
