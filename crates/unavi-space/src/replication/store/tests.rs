use hsd::{
    id::PrimId,
    property::name::PropName,
};
use iroh_docs::NamespaceSecret;
use unavi_identity::identity::Identity;
use unavi_local::DeviceStorage;
use unavi_policy::{
    quota::limits::Limits,
    trust::TrustTable,
};
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::{
    holds::MAX_HOLDS_PER_PEER,
    *,
};
use crate::replication::{
    message::{
        ClearedKey,
        HoldClaim,
        Live,
        MAX_BATCHES_PER_LAYER,
        RetainedBatch,
        SessionBatch,
        SessionWrite,
    },
    record::PinRecord,
    session::{
        SessionKey,
        layers::MAX_LAYERS_PER_DOC,
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

fn prim() -> PrimId {
    PrimId([1; 16])
}

fn name(field: &str) -> PropName {
    format!("state/{field}").parse().expect("valid prop name")
}

fn write(field: &str, value: &[u8]) -> SessionWrite {
    SessionWrite {
        prim:  prim(),
        name:  name(field).to_string(),
        value: Some(value.to_vec()),
    }
}

fn key(field: &str) -> SessionKey {
    SessionKey {
        prim: prim(),
        name: name(field),
    }
}

/// A batch from `writer`, signed by `who`, whose layer it names.
fn batch(
    writer: EndpointId,
    who: &Identity,
    doc: DocId,
    space: SpaceId,
    at: u64,
    body: Vec<SessionWrite>,
) -> Batch {
    Batch::sign(
        SessionBatch {
            writer,
            did: who.did().clone(),
            doc,
            space,
            at,
            writes: body,
            clears: Vec::new(),
        },
        who.signing_key(),
    )
    .expect("sign")
}

/// A batch dropping `writer`'s opinions on `fields`.
fn clearing(
    writer: EndpointId,
    who: &Identity,
    doc: DocId,
    space: SpaceId,
    at: u64,
    fields: &[&str],
) -> Batch {
    Batch::sign(
        SessionBatch {
            writer,
            did: who.did().clone(),
            doc,
            space,
            at,
            writes: Vec::new(),
            clears: fields
                .iter()
                .map(|field| ClearedKey {
                    prim: prim(),
                    name: name(field).to_string(),
                })
                .collect(),
        },
        who.signing_key(),
    )
    .expect("sign")
}

/// `writer` states `writes` over its own connection.
fn say(
    replicas: &Replicas,
    writer: EndpointId,
    who: &Identity,
    doc: DocId,
    space: SpaceId,
    at: u64,
    body: Vec<SessionWrite>,
) -> Result<Applied, SessionError> {
    replicas.add_session(writer, &batch(writer, who, doc, space, at, body), None)
}

/// The retained batches `writer`'s layer would be relayed as.
fn relayed_layer(replicas: &Replicas, writer: EndpointId) -> Vec<RetainedBatch> {
    let feed = replicas.register_stream(writer);
    let Ok(ReplicationMsg::Snapshot { docs, .. }) = feed.rx.try_recv() else {
        panic!("a stream opens with a snapshot");
    };
    replicas.unregister_stream(feed.token);
    docs.into_iter()
        .flat_map(|doc| doc.layers)
        .flat_map(|layer| layer.batches)
        .collect()
}

/// A relayed batch as its receiver decodes it.
fn decoded(retained: &RetainedBatch) -> Batch {
    let payload = retained.batch.payload().expect("decode");
    Batch {
        stamp: payload.at,
        payload,
        signed: retained.batch.clone(),
    }
}

/// Applies only the writes at `writes`, as a relayer's retained batch does.
const fn only(writes: &[u16]) -> Live<'_> {
    Live {
        writes,
        clears: &[],
    }
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

    assert!(
        !replicas.remove_pin(phone, doc),
        "the laptop still pins as author"
    );
    assert_eq!(replicas.author_endpoint(space, doc), Some(laptop));
    assert!(replicas.remove_pin(laptop, doc));
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
fn each_writers_opinion_on_a_key_lands_in_its_own_layer() {
    let replicas = replicas();
    let space = space(b"own-layers");
    let doc = space.doc();
    let (first, second) = (peer(1), peer(2));
    let (alice, bob) = (identity(), identity());

    say(
        &replicas,
        first,
        &alice,
        doc,
        space,
        1,
        vec![write("k", b"first")],
    )
    .expect("first writes");
    say(
        &replicas,
        second,
        &bob,
        doc,
        space,
        2,
        vec![write("k", b"second")],
    )
    .expect("second writes");

    let mut writers = replicas.session_writers(doc);
    writers.sort_unstable();
    let mut expected = vec![first, second];
    expected.sort_unstable();
    assert_eq!(writers, expected);
    assert_eq!(
        replicas.session_opinions(first, doc),
        vec![(key("k"), Some(b"first".to_vec()), 1)],
        "a later writer must not overwrite another peer's opinion"
    );
    assert_eq!(
        replicas.session_opinions(second, doc),
        vec![(key("k"), Some(b"second".to_vec()), 2)]
    );
}

#[test]
fn within_one_layer_an_older_opinion_is_skipped() {
    let replicas = replicas();
    let space = space(b"older");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        5,
        vec![write("k", b"new")],
    )
    .expect("write");
    let applied = say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        3,
        vec![write("k", b"old")],
    )
    .expect("a stale write is not an error");
    assert!(
        applied.set.is_empty(),
        "a skipped opinion is not reported changed"
    );
    assert_eq!(
        replicas.session_opinions(writer, doc),
        vec![(key("k"), Some(b"new".to_vec()), 5)]
    );
}

#[test]
fn a_batch_refused_by_quota_leaves_nothing_behind() {
    let replicas = replicas();
    let space = space(b"atomic");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());
    let batch = vec![
        write("small", b"v"),
        write("huge", &vec![0; 9 * 1024 * 1024]),
    ];

    assert_eq!(
        say(&replicas, writer, &alice, doc, space, 1, batch).err(),
        Some(SessionError::QuotaExceeded)
    );
    assert!(replicas.session_writers(doc).is_empty(), "no layer");
    assert!(
        replicas.session_opinions(writer, doc).is_empty(),
        "no opinion"
    );
    let quota = replicas
        .attribution()
        .writer_quota(writer, Some(alice.did()));
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
    let attribution = replicas.attribution();
    let author_documents = attribution
        .author_quota(alice.did())
        .usage(Stock::Documents);

    say(
        &replicas,
        peer(2),
        &guest,
        doc,
        space,
        2,
        vec![write("k", &[0; 64])],
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
    assert_eq!(
        say(
            &replicas,
            peer(2),
            &identity(),
            doc,
            bogus,
            2,
            vec![write("k", b"v")]
        )
        .err(),
        Some(SessionError::WrongSpace),
        "a document present in one space is not writable through another"
    );
    assert!(
        replicas.session_writers(doc).is_empty(),
        "a refused write must not stack a layer"
    );
}

#[test]
fn a_document_holds_no_more_layers_than_the_cap() {
    let replicas = replicas();
    let space = space(b"cap");
    let doc = space.doc();

    let writers = (1..=MAX_LAYERS_PER_DOC as u8 + 1)
        .map(|seed| (peer(seed), identity()))
        .collect::<Vec<_>>();
    let (past, past_who) = writers.last().expect("a writer past the cap");
    for (writer, who) in &writers[..MAX_LAYERS_PER_DOC] {
        say(
            &replicas,
            *writer,
            who,
            doc,
            space,
            1,
            vec![write("k", b"v")],
        )
        .expect("under the cap");
    }
    assert_eq!(
        say(
            &replicas,
            *past,
            past_who,
            doc,
            space,
            1,
            vec![write("k", b"v")]
        )
        .err(),
        Some(SessionError::QuotaExceeded)
    );
    assert!(
        replicas.session_opinions(*past, doc).is_empty(),
        "the refused peer holds no layer"
    );
    let (first, first_who) = &writers[0];
    assert!(
        say(
            &replicas,
            *first,
            first_who,
            doc,
            space,
            2,
            vec![write("k", b"w")]
        )
        .is_ok(),
        "a peer already holding a layer is not refused by the cap"
    );
}

#[test]
fn a_layer_is_removed_only_when_its_last_source_goes() {
    let replicas = replicas();
    let space = space(b"release");
    let doc = space.doc();
    let (writer, relayer, alice) = (peer(1), peer(2), identity());
    let quota = replicas
        .attribution()
        .writer_quota(writer, Some(alice.did()));

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        1,
        vec![write("k", b"v")],
    )
    .expect("write");
    assert!(replicas.add_session_source(writer, doc, relayer));
    assert_eq!(quota.usage(Stock::Documents), 1);
    assert!(quota.usage(Stock::SessionMemory) > 0);

    assert!(
        !replicas.remove_session_source(writer, doc, writer),
        "a source remains"
    );
    assert_eq!(replicas.session_writers(doc), vec![writer]);
    assert_eq!(replicas.session_opinions(writer, doc).len(), 1);

    assert!(replicas.remove_session_source(writer, doc, relayer));
    assert!(
        replicas.session_writers(doc).is_empty(),
        "the layer is gone"
    );
    assert_eq!(quota.usage(Stock::Documents), 0);
    assert_eq!(quota.usage(Stock::SessionMemory), 0);
}

#[test]
fn a_relayed_layer_for_an_endpoint_held_under_another_did_is_refused() {
    let replicas = replicas();
    let space = space(b"wrong-did");
    let doc = space.doc();
    let (alice, mallory) = (identity(), identity());

    say(
        &replicas,
        peer(1),
        &alice,
        doc,
        space,
        1,
        vec![write("k", b"v")],
    )
    .expect("write");
    let forged = batch(
        peer(1),
        &mallory,
        doc,
        space,
        2,
        vec![write("k", b"forged")],
    );
    assert_eq!(
        replicas.add_session(peer(2), &forged, None).err(),
        Some(SessionError::WrongDid)
    );
    assert_eq!(
        replicas.session_opinions(peer(1), doc),
        vec![(key("k"), Some(b"v".to_vec()), 1)]
    );
    assert_eq!(replicas.session_sources(peer(1), doc), vec![peer(1)]);
}

#[test]
fn a_live_filter_keeps_a_cleared_key_from_returning() {
    let space = space(b"live");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());
    let batch = batch(
        writer,
        &alice,
        doc,
        space,
        1,
        vec![write("a", b"1"), write("b", b"2")],
    );

    let held = Replicas::new(Policy::new(), Rules::default());
    held.add_session(writer, &batch, None).expect("held");
    let cleared = clearing(writer, &alice, doc, space, 2, &["a"]);
    held.add_session(writer, &cleared, None).expect("cleared");
    let relayed = relayed_layer(&held, writer);
    let [written, clearing] = &relayed[..] else {
        panic!("the write and the clear are retained");
    };
    assert_eq!(written.live, vec![1], "only the write still current");
    assert_eq!(clearing.live_clears, vec![0], "the clear is relayed too");

    let receiver = Replicas::new(Policy::new(), Rules::default());
    for retained in &relayed {
        receiver
            .add_session(peer(9), &decoded(retained), Some(retained.live()))
            .expect("relayed");
    }
    assert_eq!(
        receiver.session_opinions(writer, doc),
        vec![(key("b"), Some(b"2".to_vec()), 1)],
        "the cleared key stays cleared"
    );
    assert_eq!(
        receiver.session_tombstones(writer, doc),
        vec![(key("a"), 2)]
    );
}

#[test]
fn a_retained_batch_goes_once_no_cell_points_at_it() {
    let replicas = replicas();
    let space = space(b"retained");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());
    let quota = replicas
        .attribution()
        .writer_quota(writer, Some(alice.did()));

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        1,
        vec![write("a", b"1"), write("b", b"2")],
    )
    .expect("first");
    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        2,
        vec![write("a", b"3")],
    )
    .expect("second");
    assert_eq!(
        relayed_layer(&replicas, writer).len(),
        2,
        "the first batch still backs b"
    );
    let both = quota.usage(Stock::SessionMemory);

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        3,
        vec![write("b", b"4")],
    )
    .expect("third");
    let layer = relayed_layer(&replicas, writer);
    assert_eq!(layer.len(), 2, "the first batch lost its last cell");
    assert!(layer.iter().all(|retained| !retained.live.is_empty()));
    assert!(
        quota.usage(Stock::SessionMemory) < both,
        "a dropped batch releases its charge"
    );

    replicas
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 4, &["a", "b"]),
            None,
        )
        .expect("clear");
    let layer = relayed_layer(&replicas, writer);
    assert_eq!(layer.len(), 1, "only the clear is left");
    assert_eq!(layer[0].live_clears, vec![0, 1]);

    assert!(replicas.remove_session_source(writer, doc, writer));
    assert_eq!(quota.usage(Stock::SessionMemory), 0);
}

/// Leaves a layer holding 30 KB of batches for one tiny live cell.
fn leave_dead_weight(
    replicas: &Replicas,
    writer: EndpointId,
    who: &Identity,
    doc: DocId,
    space: SpaceId,
    prefix: &str,
) {
    let key = |letter: &str| format!("{prefix}{letter}");
    let fat = |letter: &str| write(&key(letter), &[0; 10_000]);
    say(
        replicas,
        writer,
        who,
        doc,
        space,
        1,
        vec![fat("a"), fat("b"), fat("c"), write(&key("d"), b"1")],
    )
    .expect("first");
    let thin = |letter: &str| write(&key(letter), b"2");
    say(
        replicas,
        writer,
        who,
        doc,
        space,
        2,
        vec![thin("a"), thin("b"), thin("c")],
    )
    .expect("second");
}

#[test]
fn a_layer_is_due_a_checkpoint_once_its_batches_outgrow_its_content() {
    let replicas = replicas();
    let space = space(b"checkpoint");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());

    assert!(!replicas.needs_checkpoint(writer, doc), "no layer yet");
    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        1,
        vec![write("a", &[0; 10_000])],
    )
    .expect("first");
    assert!(!replicas.needs_checkpoint(writer, doc), "nothing is stale");

    leave_dead_weight(&replicas, writer, &alice, doc, space, "x");
    assert!(
        replicas.needs_checkpoint(writer, doc),
        "the first batches hold 30 KB for tiny cells"
    );
}

#[test]
fn a_failed_checkpoint_waits_for_the_layer_to_grow() {
    let replicas = replicas();
    let space = space(b"backoff");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());

    leave_dead_weight(&replicas, writer, &alice, doc, space, "x");
    assert!(replicas.needs_checkpoint(writer, doc));

    replicas.record_checkpoint(writer, doc, false);
    assert!(!replicas.needs_checkpoint(writer, doc), "backed off");

    leave_dead_weight(&replicas, writer, &alice, doc, space, "y");
    assert!(
        replicas.needs_checkpoint(writer, doc),
        "the layer grew past the failed attempt"
    );

    replicas.record_checkpoint(writer, doc, true);
    assert!(
        replicas.needs_checkpoint(writer, doc),
        "a success clears the back-off"
    );
}

#[test]
fn a_layer_refuses_a_batch_past_the_retained_cap() {
    let replicas = replicas();
    let space = space(b"retained-cap");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());

    for i in 0..MAX_BATCHES_PER_LAYER {
        say(
            &replicas,
            writer,
            &alice,
            doc,
            space,
            1,
            vec![write(&format!("k{i}"), b"v")],
        )
        .expect("under the cap");
    }
    assert_eq!(
        say(
            &replicas,
            writer,
            &alice,
            doc,
            space,
            2,
            vec![write("more", b"v")]
        )
        .err(),
        Some(SessionError::QuotaExceeded),
        "a layer too large to relay is not grown"
    );
    assert_eq!(
        replicas
            .add_session(
                writer,
                &clearing(writer, &alice, doc, space, 3, &["k0"]),
                None
            )
            .err(),
        Some(SessionError::QuotaExceeded),
        "a clear is a retained batch too"
    );
}

#[test]
fn the_last_write_to_a_key_in_a_batch_wins_and_leaks_nothing() {
    let replicas = replicas();
    let space = space(b"duplicate");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());
    let quota = replicas
        .attribution()
        .writer_quota(writer, Some(alice.did()));

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        1,
        vec![write("k", b"first"), write("k", b"second")],
    )
    .expect("written");
    assert_eq!(
        replicas.session_opinions(writer, doc),
        vec![(key("k"), Some(b"second".to_vec()), 1)]
    );
    assert_eq!(relayed_layer(&replicas, writer)[0].live, vec![1]);

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        2,
        vec![write("k", b"third")],
    )
    .expect("overwrite");
    assert_eq!(
        relayed_layer(&replicas, writer).len(),
        1,
        "the first batch lost its last cell"
    );
    replicas
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 3, &["k"]),
            None,
        )
        .expect("clear");
    assert!(replicas.remove_session_source(writer, doc, writer));
    assert_eq!(quota.usage(Stock::SessionMemory), 0);
}

#[test]
fn a_relayer_holds_a_layer_only_for_a_write_that_is_current() {
    let replicas = replicas();
    let space = space(b"relayer");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());
    let live = [0];

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        5,
        vec![write("k", b"new")],
    )
    .expect("written");

    let stale = batch(writer, &alice, doc, space, 3, vec![write("k", b"old")]);
    replicas
        .add_session(peer(2), &stale, Some(only(&live)))
        .expect("stale");
    assert_eq!(
        replicas.session_sources(writer, doc),
        vec![writer],
        "an older opinion earns no source"
    );

    let current = batch(writer, &alice, doc, space, 5, vec![write("k", b"new")]);
    replicas
        .add_session(peer(3), &current, Some(only(&[])))
        .expect("nothing live");
    assert_eq!(replicas.session_sources(writer, doc), vec![writer]);

    replicas
        .add_session(peer(3), &current, Some(only(&live)))
        .expect("current");
    let mut sources = replicas.session_sources(writer, doc);
    sources.sort_unstable();
    let mut expected = vec![writer, peer(3)];
    expected.sort_unstable();
    assert_eq!(
        sources, expected,
        "the same opinion is held by another source"
    );
}

#[test]
fn rewriting_a_cell_with_the_same_value_reports_no_change() {
    let replicas = replicas();
    let space = space(b"unchanged");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        1,
        vec![write("k", b"v")],
    )
    .expect("written");
    let applied = say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        2,
        vec![write("k", b"v")],
    )
    .expect("rewritten");
    assert!(
        applied.set.is_empty(),
        "the composed value is already current"
    );
    assert_eq!(
        replicas.session_opinions(writer, doc),
        vec![(key("k"), Some(b"v".to_vec()), 2)],
        "the stamp still moves"
    );
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
        say(
            &replicas,
            peer(seed),
            &identity(),
            doc,
            space,
            1,
            vec![write("k", b"v")],
        )
        .expect("under the cap");
    }
    assert!(
        say(
            &replicas,
            peer(200),
            &author,
            doc,
            space,
            2,
            vec![write("k", b"v")]
        )
        .is_ok(),
        "a crowd of strangers must not lock the author out of its own document"
    );
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

    assert!(replicas.remove_pin(author_device, doc));
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

#[test]
fn a_tombstone_beats_a_dishonest_relayers_older_batch_in_either_arrival_order() {
    let space = space(b"tombstone-order");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());
    let old = batch(writer, &alice, doc, space, 1, vec![write("k", b"old")]);

    let honest = Replicas::new(Policy::new(), Rules::default());
    honest.add_session(writer, &old, None).expect("written");
    honest
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 2, &["k"]),
            None,
        )
        .expect("cleared");
    let relayed = relayed_layer(&honest, writer);
    let [tombstone] = &relayed[..] else {
        panic!("the clear is all an honest carrier retains");
    };
    assert_eq!(tombstone.live, Vec::<u16>::new());
    assert_eq!(tombstone.live_clears, vec![0]);

    for tombstone_first in [false, true] {
        let receiver = Replicas::new(Policy::new(), Rules::default());
        let stale = || {
            receiver
                .add_session(peer(8), &old, Some(only(&[0])))
                .expect("relayed");
        };
        let fresh = || {
            receiver
                .add_session(peer(9), &decoded(tombstone), Some(tombstone.live()))
                .expect("relayed");
        };
        if tombstone_first {
            fresh();
            stale();
        } else {
            stale();
            fresh();
        }
        assert_eq!(
            receiver.session_opinions(writer, doc),
            Vec::new(),
            "the cleared key stays cleared (tombstone first: {tombstone_first})"
        );
        assert_eq!(
            receiver.session_tombstones(writer, doc),
            vec![(key("k"), 2)]
        );
    }
}

#[test]
fn a_write_newer_than_a_tombstone_wins_and_an_equal_stamp_does_not() {
    let replicas = replicas();
    let space = space(b"tombstone-order-by-stamp");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        1,
        vec![write("k", b"v1")],
    )
    .expect("written");
    replicas
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 2, &["k"]),
            None,
        )
        .expect("cleared");

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        2,
        vec![write("k", b"tie")],
    )
    .expect("tie");
    assert_eq!(
        replicas.session_opinions(writer, doc),
        Vec::new(),
        "a tie goes to the clear"
    );

    let applied = say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        3,
        vec![write("k", b"v3")],
    )
    .expect("newer");
    assert_eq!(
        applied.set.len(),
        1,
        "the write composes over the tombstone"
    );
    assert_eq!(
        replicas.session_opinions(writer, doc),
        vec![(key("k"), Some(b"v3".to_vec()), 3)]
    );
    assert_eq!(replicas.session_tombstones(writer, doc), Vec::new());
}

#[test]
fn a_clear_reports_the_composed_opinion_it_dropped_only() {
    let replicas = replicas();
    let space = space(b"tombstone-report");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());

    let first = replicas
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 1, &["k"]),
            None,
        )
        .expect("a clear with nothing to drop");
    assert!(
        first.cleared.is_empty() && first.held,
        "it is held all the same"
    );

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        2,
        vec![write("k", b"v")],
    )
    .expect("written");
    let second = replicas
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 3, &["k"]),
            None,
        )
        .expect("cleared");
    assert_eq!(second.cleared, vec![key("k")]);
    let third = replicas
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 4, &["k"]),
            None,
        )
        .expect("cleared again");
    assert!(
        third.cleared.is_empty(),
        "a tombstone has nothing composed to drop"
    );
}

#[test]
fn retained_batches_and_leases_balance_across_tombstones() {
    let replicas = replicas();
    let space = space(b"tombstone-leases");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());
    let quota = replicas
        .attribution()
        .writer_quota(writer, Some(alice.did()));

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        1,
        vec![write("k", b"v")],
    )
    .expect("written");
    replicas
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 2, &["k"]),
            None,
        )
        .expect("cleared");
    let layer = relayed_layer(&replicas, writer);
    assert_eq!(
        layer.len(),
        1,
        "the tombstone replaced the write's only claim on its batch"
    );
    assert_eq!(layer[0].live_clears, vec![0]);
    assert_eq!(layer[0].live, Vec::<u16>::new());
    let with_tombstone = quota.usage(Stock::SessionMemory);
    assert!(with_tombstone > 0, "a tombstone is charged");

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        3,
        vec![write("k", b"w")],
    )
    .expect("rewritten");
    let layer = relayed_layer(&replicas, writer);
    assert_eq!(layer.len(), 1, "the write took over from the tombstone");
    assert_eq!(layer[0].live, vec![0]);
    assert_eq!(layer[0].live_clears, Vec::<u16>::new());

    replicas
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 4, &["k"]),
            None,
        )
        .expect("cleared again");
    assert!(replicas.remove_session_source(writer, doc, writer));
    assert_eq!(
        quota.usage(Stock::SessionMemory),
        0,
        "every lease is released"
    );
    assert_eq!(quota.usage(Stock::Documents), 0);
}

#[test]
fn a_relayer_holds_a_layer_only_for_a_clear_that_is_current() {
    let replicas = replicas();
    let space = space(b"tombstone-relayer");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());
    let clears = Some(Live {
        writes: &[],
        clears: &[0],
    });

    replicas
        .add_session(
            writer,
            &clearing(writer, &alice, doc, space, 5, &["k"]),
            None,
        )
        .expect("cleared");

    let stale = clearing(writer, &alice, doc, space, 3, &["k"]);
    replicas
        .add_session(peer(2), &stale, clears)
        .expect("stale");
    assert_eq!(
        replicas.session_sources(writer, doc),
        vec![writer],
        "an older clear earns no source"
    );

    let current = clearing(writer, &alice, doc, space, 5, &["k"]);
    replicas
        .add_session(peer(3), &current, clears)
        .expect("current");
    let mut sources = replicas.session_sources(writer, doc);
    sources.sort_unstable();
    let mut expected = vec![writer, peer(3)];
    expected.sort_unstable();
    assert_eq!(sources, expected);
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
    assert!(replicas.remove_pin(peer(1), doc));
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

    assert!(replicas.remove_pin(peer(1), doc));
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
    assert!(replicas.remove_pin(peer(1), doc));

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
        let feed = replicas.register_stream(peer(1));
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

    let feed = replicas.register_stream(me);
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

#[test]
fn a_writer_at_its_quota_can_still_clear() {
    let replicas = replicas();
    let space = space(b"clear-at-quota");
    let doc = space.doc();
    let (writer, alice) = (peer(1), identity());
    let quota = replicas
        .attribution()
        .writer_quota(writer, Some(alice.did()));

    say(
        &replicas,
        writer,
        &alice,
        doc,
        space,
        1,
        vec![write("k", &[0; 200])],
    )
    .expect("written");
    let clear = clearing(writer, &alice, doc, space, 2, &["k"]);
    let mut limits = Limits::default();
    limits.stock.insert(
        Stock::SessionMemory,
        quota.usage(Stock::SessionMemory) + clear.signed.payload_len() as u64,
    );
    quota.set_limits(limits);

    replicas
        .add_session(writer, &clear, None)
        .expect("the tombstone takes over the lease of the cell it replaces");
    assert_eq!(
        replicas.session_tombstones(writer, doc),
        vec![(key("k"), 2)]
    );
}
