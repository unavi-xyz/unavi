use hsd::{
    id::PrimId,
    property::name::PropName,
};
use iroh_docs::NamespaceSecret;
use unavi_identity::identity::Identity;
use unavi_local::DeviceStorage;
use unavi_policy::trust::TrustTable;
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;
use crate::replication::session::layers::MAX_LAYERS_PER_DOC;

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
    format!("state/{field}").parse().expect("valid prop name")
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
fn anyone_may_hold_until_the_holder_reserves_the_next() {
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
        replicas.add_hold(guest, Some(&guest_did), doc, space, 2),
        "anyone may take physics authority by default"
    );
    assert_eq!(replicas.holder(space, doc), Some(guest));

    replicas.remove_hold(guest, doc, Some(other));
    assert!(!replicas.add_hold(guest, Some(&guest_did), doc, space, 3));
    assert!(replicas.add_hold(other, None, doc, space, 4));
    assert_eq!(replicas.holder(space, doc), Some(other));

    replicas.remove_hold(other, doc, None);
    assert_eq!(replicas.holder(space, doc), Some(author_device));
    assert!(
        replicas.add_hold(guest, Some(&guest_did), doc, space, 5),
        "a spent reservation opens the document again"
    );

    replicas.remove_hold(guest, doc, Some(other));
    assert!(
        replicas.add_hold(author_device, Some(alice.did()), doc, space, 7),
        "the author holds through a reservation"
    );
    assert!(
        replicas.add_hold(other, None, doc, space, 8),
        "the author's hold clears the reservation"
    );
}

#[test]
fn each_peers_opinion_on_a_key_lands_in_its_own_layer() {
    let replicas = replicas();
    let space = space(b"own-layers");
    let doc = space.doc();
    let (first, second) = (peer(1), peer(2));
    let key = SessionKey {
        prim: prim(),
        name: name("k"),
    };

    replicas
        .add_session(first, None, doc, space, write("k", b"first"), 1)
        .expect("first writes");
    replicas
        .add_session(second, None, doc, space, write("k", b"second"), 2)
        .expect("second writes");

    let mut peers = replicas.session_peers(doc);
    peers.sort_unstable();
    let mut expected = vec![first, second];
    expected.sort_unstable();
    assert_eq!(peers, expected);
    assert_eq!(
        replicas.session_opinions(first, doc),
        vec![(key.clone(), Some(b"first".to_vec()), 1)],
        "a later writer must not overwrite another peer's opinion"
    );
    assert_eq!(
        replicas.session_opinions(second, doc),
        vec![(key, Some(b"second".to_vec()), 2)]
    );
}

#[test]
fn within_one_layer_an_older_opinion_is_skipped() {
    let replicas = replicas();
    let space = space(b"older");
    let doc = space.doc();
    let writer = peer(1);
    let key = SessionKey {
        prim: prim(),
        name: name("k"),
    };

    replicas
        .add_session(writer, None, doc, space, write("k", b"new"), 5)
        .expect("write");
    let changed = replicas
        .add_session(writer, None, doc, space, write("k", b"old"), 3)
        .expect("a stale write is not an error");
    assert!(
        changed.is_empty(),
        "a skipped opinion is not reported changed"
    );
    assert_eq!(
        replicas.session_opinions(writer, doc),
        vec![(key, Some(b"new".to_vec()), 5)]
    );
}

#[test]
fn a_batch_refused_by_quota_leaves_nothing_behind() {
    let replicas = replicas();
    let space = space(b"atomic");
    let doc = space.doc();
    let writer = peer(1);
    let mut batch = write("small", b"v");
    batch.push((
        SessionKey {
            prim: prim(),
            name: name("huge"),
        },
        Some(vec![0; 9 * 1024 * 1024]),
    ));

    assert_eq!(
        replicas.add_session(writer, None, doc, space, batch, 1),
        Err(SessionError::QuotaExceeded)
    );
    assert!(replicas.session_peers(doc).is_empty(), "no layer");
    assert!(
        replicas.session_opinions(writer, doc).is_empty(),
        "no opinion"
    );
    let quota = replicas.attribution().writer_quota(writer, None);
    assert_eq!(quota.usage(Stock::Documents), 0, "no layer charge");
    assert_eq!(quota.usage(Stock::SessionMemory), 0, "no opinion charge");
}

#[test]
fn the_writer_pays_for_its_layer_on_an_authored_document() {
    let replicas = replicas();
    let (alice, guest) = (identity(), identity());
    let (doc, proof) = authored(10, &alice);
    let space = space(b"charge");

    replicas
        .add_pin(peer(1), doc, space, alice.did().clone(), proof, 1)
        .expect("pin");
    let attribution = replicas.attribution();
    let author_documents = attribution
        .author_quota(alice.did())
        .usage(Stock::Documents);

    replicas
        .add_session(
            peer(2),
            Some(guest.did()),
            doc,
            space,
            write("k", &[0; 64]),
            2,
        )
        .expect("write");

    let guest_quota = attribution.author_quota(guest.did());
    assert_eq!(guest_quota.usage(Stock::Documents), 1);
    assert!(guest_quota.usage(Stock::SessionMemory) > 0);
    let author_quota = attribution.author_quota(alice.did());
    assert_eq!(
        author_quota.usage(Stock::SessionMemory),
        0,
        "a guest writing to the author's document must not spend the author's budget"
    );
    assert_eq!(
        author_quota.usage(Stock::Documents),
        author_documents,
        "the guest's layer is not a document of the author's"
    );
}

#[test]
fn a_write_naming_another_space_than_the_documents_is_refused() {
    let replicas = replicas();
    let alice = identity();
    let (doc, proof) = authored(5, &alice);
    let (real, bogus) = (space(b"real-space"), space(b"bogus-space"));

    replicas
        .add_pin(peer(1), doc, real, alice.did().clone(), proof, 1)
        .expect("pin");
    assert_eq!(
        replicas.add_session(peer(2), None, doc, bogus, write("k", b"v"), 2),
        Err(SessionError::WrongSpace),
        "a document present in one space is not writable through another"
    );
    assert!(
        replicas.session_peers(doc).is_empty(),
        "a refused write must not stack a layer"
    );
}

#[test]
fn a_document_holds_no_more_layers_than_the_cap() {
    let replicas = replicas();
    let space = space(b"cap");
    let doc = space.doc();

    for seed in 1..=MAX_LAYERS_PER_DOC as u8 {
        replicas
            .add_session(peer(seed), None, doc, space, write("k", b"v"), 1)
            .expect("under the cap");
    }
    let past = peer(MAX_LAYERS_PER_DOC as u8 + 1);
    assert_eq!(
        replicas.add_session(past, None, doc, space, write("k", b"v"), 1),
        Err(SessionError::QuotaExceeded)
    );
    assert!(
        replicas.session_opinions(past, doc).is_empty(),
        "the refused peer holds no layer"
    );
    assert!(
        replicas
            .add_session(peer(1), None, doc, space, write("k", b"w"), 2)
            .is_ok(),
        "a peer already holding a layer is not refused by the cap"
    );
}

#[test]
fn removing_a_layer_releases_its_charge() {
    let replicas = replicas();
    let space = space(b"release");
    let doc = space.doc();
    let writer = peer(1);
    let quota = replicas.attribution().writer_quota(writer, None);

    replicas
        .add_session(writer, None, doc, space, write("k", b"v"), 1)
        .expect("write");
    assert_eq!(quota.usage(Stock::Documents), 1);
    assert!(quota.usage(Stock::SessionMemory) > 0);

    replicas.remove_session_layer(writer, doc);
    assert!(replicas.session_peers(doc).is_empty(), "the layer is gone");
    assert_eq!(quota.usage(Stock::Documents), 0);
    assert_eq!(quota.usage(Stock::SessionMemory), 0);
}

#[test]
fn a_reference_is_judged_by_its_hosts_author() {
    let policy = Policy::new();
    let replicas = Replicas::new(policy.clone(), Rules::default());
    let alice = identity();
    let (host, proof) = authored(8, &alice);
    let reference = DocId([9; 32]);
    let space = space(b"host-chain");
    policy.update(reference, |record| record.host = Some(host));

    replicas
        .add_pin(peer(1), host, space, alice.did().clone(), proof, 1)
        .expect("pin");
    assert_eq!(
        replicas.author(reference).as_ref(),
        Some(alice.did()),
        "the host's author authors what it references"
    );
}

#[test]
fn a_pin_refused_by_quota_leaves_no_quota_behind() {
    let policy = Policy::new();
    let replicas = Replicas::new(policy.clone(), Rules::default());
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

#[test]
fn the_author_takes_a_reserved_layer_past_the_cap() {
    let replicas = Replicas::new(Policy::new(), Rules::default());
    let author = identity();
    replicas.attribution().set_local(
        author.did().clone(),
        TrustTable::new(DeviceStorage::memory()),
    );
    let space = space(b"reserved");
    let doc = DocId([21; 32]);
    replicas.record_minted(doc);

    for seed in 1..=MAX_LAYERS_PER_DOC as u8 {
        replicas
            .add_session(peer(seed), None, doc, space, write("k", b"v"), 1)
            .expect("under the cap");
    }
    assert!(
        replicas
            .add_session(
                peer(200),
                Some(author.did()),
                doc,
                space,
                write("k", b"v"),
                2
            )
            .is_ok(),
        "a crowd of strangers must not lock the author out of its own document"
    );
}
