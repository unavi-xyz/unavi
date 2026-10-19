use hsd::id::PrimId;
use iroh::EndpointId;
use unavi_identity::identity::Identity;
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;
use crate::replication::message::session::{
    LayerState,
    MAX_VALUE_BYTES,
    MAX_WRITES_PER_TICK,
    SessionWrite,
    SignedLayer,
};

fn identity() -> Identity {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    Identity::new(did, key)
}

fn device() -> EndpointId {
    iroh::SecretKey::from_bytes(&[5; 32]).public()
}

const DOC: DocId = DocId([1; 32]);
const SPACE: SpaceId = SpaceId([2; 32]);

fn state(who: &Identity, doc: DocId, at: u64, root: [u8; 32]) -> SignedLayer {
    SignedLayer::sign(
        LayerState {
            device: device(),
            did: who.did().clone(),
            doc,
            space: SPACE,
            at,
            root,
        },
        who.signing_key(),
    )
    .expect("sign")
}

fn write(name: &str, len: usize) -> SessionWrite {
    SessionWrite {
        prim:  PrimId([3; 16]),
        name:  name.to_owned(),
        value: Some(vec![0; len]),
    }
}

fn tick(writes: Vec<SessionWrite>) -> ReplicationMsg {
    ReplicationMsg::Session(SessionTick {
        writes,
        clears: Vec::new(),
        state: state(&identity(), DOC, 1, [0; 32]).signed,
    })
}

fn doc(layers: Vec<LayerSnapshot>) -> DocSnapshot {
    DocSnapshot {
        doc: DOC,
        space: SPACE,
        pin: None,
        hold: Some(1),
        withdrawal: None,
        layers,
    }
}

#[test]
fn a_tick_past_any_bound_is_rejected() {
    let many = (0..=MAX_WRITES_PER_TICK)
        .map(|n| write(&format!("state/k{n}"), 1))
        .collect();
    assert_eq!(tick(many).validate(0).err(), Some(Rejected::TickTooLarge));
    assert_eq!(
        tick(vec![write("state/k", MAX_VALUE_BYTES + 1)])
            .validate(0)
            .err(),
        Some(Rejected::ValueTooLarge)
    );
    let long = format!("state/{}", "k".repeat(300));
    assert_eq!(
        tick(vec![write(&long, 1)]).validate(0).err(),
        Some(Rejected::NameTooLong)
    );
    assert!(
        tick(vec![write("state/k", MAX_VALUE_BYTES)])
            .validate(0)
            .is_ok()
    );
}

#[test]
fn a_layer_naming_another_document_is_skipped_and_the_rest_applies() {
    let who = identity();
    let good = LayerSnapshot {
        state: state(&who, DOC, 1, [0; 32]).signed,
        cells: vec![write("state/k", 1)],
    };
    let elsewhere = LayerSnapshot {
        state: state(&who, DocId([9; 32]), 1, [0; 32]).signed,
        cells: Vec::new(),
    };
    let msg = ReplicationMsg::Snapshot {
        scope: None,
        docs:  vec![doc(vec![elsewhere, good])],
        more:  false,
    };
    let Ok(Received::Snapshot { docs, .. }) = msg.validate(0) else {
        panic!("a bad layer does not reject the message");
    };
    assert_eq!(docs[0].hold, Some(1), "the document's hold survives");
    assert_eq!(docs[0].layers.len(), 1, "only the good layer remains");
}

#[test]
fn pieces_of_one_layer_split_across_frames_join_back_into_it() {
    let who = identity();
    let piece = |name: &str, at| LayerSnapshot {
        state: state(&who, DOC, at, [0; 32]).signed,
        cells: vec![write(name, 1)],
    };
    let frame = |layer| {
        let Ok(Received::Snapshot { docs, .. }) = (ReplicationMsg::Snapshot {
            scope: None,
            docs:  vec![doc(vec![layer])],
            more:  true,
        })
        .validate(0) else {
            panic!("a piece validates");
        };
        docs
    };
    let docs = [
        frame(piece("state/a", 1)),
        frame(piece("state/b", 1)),
        frame(piece("state/c", 0)),
    ]
    .into_iter()
    .flatten()
    .collect();
    let merged = merge_snapshot(docs);
    assert_eq!(merged.len(), 1, "one entry per document");
    assert_eq!(merged[0].layers.len(), 1, "one layer per writer");
    let names = merged[0].layers[0]
        .cells
        .iter()
        .map(|(key, _)| key.name.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec!["state/a", "state/b"],
        "a piece stating an older state is not mixed in"
    );
}

#[test]
fn a_hold_stamped_past_the_skew_is_refused() {
    let recv = 1_000_000_000;
    let hold = |at| {
        ReplicationMsg::Hold(HoldClaim {
            doc: DOC,
            space: SPACE,
            at,
        })
    };
    let limit = recv + clock::MAX_SKEW_MICROS;
    assert!(hold(limit).validate(recv).is_ok());
    assert_eq!(
        hold(limit + 1).validate(recv).err(),
        Some(Rejected::FutureHold)
    );

    let mut snapshot = doc(Vec::new());
    snapshot.hold = Some(u64::MAX);
    let msg = ReplicationMsg::Snapshot {
        scope: None,
        docs:  vec![snapshot],
        more:  false,
    };
    let Ok(Received::Snapshot { docs, .. }) = msg.validate(recv) else {
        panic!("a future hold costs the document only its hold");
    };
    assert_eq!(docs[0].hold, None);
}
