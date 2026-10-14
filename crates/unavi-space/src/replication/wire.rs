//! The state stream: length-prefixed [`ReplicationMsg`] frames, checked and
//! proven before they reach the world.

use std::time::Duration;

use anyhow::{
    Context,
    bail,
};
use bevy::prelude::*;
use iroh::{
    EndpointId,
    endpoint::{
        Connection,
        RecvStream,
        SendStream,
    },
};
use iroh_docs::NamespaceId;
use n0_future::time::timeout;
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use tracing::warn;
use unavi_identity::{
    authorship::Authorship,
    resolver::Resolver,
};
use xdid::core::{
    did::Did,
    document::Document,
};

use crate::{
    link::{
        PeerLink,
        streams::{
            StreamKind,
            read_disconnected,
        },
    },
    membership::SpaceId,
    replication::{
        clock,
        frames::{
            self,
            MAX_SNAPSHOT_FRAMES,
            MAX_STATE_FRAME,
        },
        guards::{
            self,
            Remote,
            RemoteDoc,
            pins::PinClaim,
        },
        message::{
            Batch,
            Received,
            ReceivedDoc,
            ReplicationMsg,
            Retained,
        },
        record::Record,
        store::Proven,
    },
};

/// Bytes allocated ahead of a frame's content; the rest grows as it arrives,
/// so a length prefix alone never costs a full frame.
const READ_CHUNK: usize = 16 * 1024;

/// How long a DID may take to resolve before the batches naming it are
/// dropped.
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn send_state_stream(link: &PeerLink, connection: &Connection) -> anyhow::Result<()> {
    let mut tx = connection.open_uni().await?;
    StreamKind::State.write(&mut tx).await?;

    let me = link.view().me();
    let replicas = link.view().replicas();
    let feed = replicas.register_stream(me);
    let res = async {
        while let Ok(msg) = feed.rx.recv().await {
            let msg = if feed.resync.load(std::sync::atomic::Ordering::Relaxed) {
                replicas.resync(&feed, me)
            } else {
                msg
            };
            for frame in frames::split(msg, MAX_STATE_FRAME) {
                write_frame(&mut tx, &frame).await?;
            }
        }
        anyhow::Ok(())
    }
    .await;
    replicas.unregister_stream(feed.token);
    res
}

async fn write_frame(tx: &mut SendStream, msg: &ReplicationMsg) -> anyhow::Result<()> {
    let buf = postcard::to_allocvec(msg)?;
    if buf.len() > MAX_STATE_FRAME {
        warn!(len = buf.len(), "state message too large to send; dropped");
        return Ok(());
    }
    tx.write_u32(u32::try_from(buf.len())?).await?;
    tx.write_all(&buf).await?;
    Ok(())
}

pub async fn recv_state_stream(
    link: &PeerLink,
    peer: EndpointId,
    did: Option<Did>,
    mut rx: RecvStream,
) -> anyhow::Result<()> {
    // Racing connections to the same peer share one state entity; the
    // generation lets only the latest stream tear it down.
    let generation = link.next_stream_gen();
    let claimed = did.clone();
    let peer_ent = link
        .view()
        .commands()
        .send_with(move |world: &mut World| {
            guards::claim_remote_peer(world, peer, claimed, generation)
        })
        .await
        .context("claim remote peer")?;

    let res = recv_loop(link, peer_ent, peer, did.as_ref(), &mut rx).await;
    let released = link
        .view()
        .commands()
        .push(move |world: &mut World| {
            guards::release_remote_peer(world, peer_ent, generation);
        })
        .send()
        .await;
    if released.is_err() {
        debug!(%peer, "the world is gone; nothing to release");
    }
    res
}

async fn recv_loop(
    link: &PeerLink,
    peer_ent: Entity,
    peer: EndpointId,
    did: Option<&Did>,
    rx: &mut RecvStream,
) -> anyhow::Result<()> {
    // A snapshot split across frames is applied once its last frame arrives,
    // so the world never sees the sender's state half restated.
    let mut snapshot = Vec::new();
    let mut snapshot_frames = 0;
    loop {
        let len = match rx.read_u32().await {
            Ok(len) => len as usize,
            Err(err) if read_disconnected(&err) => return Ok(()),
            Err(err) => return Err(err).context("read len"),
        };
        if len > MAX_STATE_FRAME {
            bail!("state frame of {len} bytes exceeds {MAX_STATE_FRAME}")
        }

        let mut buf = Vec::with_capacity(len.min(READ_CHUNK));
        (&mut *rx)
            .take(len as u64)
            .read_to_end(&mut buf)
            .await
            .context("read msg")?;
        if buf.len() != len {
            bail!("state frame truncated");
        }

        let msg = postcard::from_bytes::<ReplicationMsg>(&buf).context("parse msg")?;
        let is_snapshot = matches!(msg, ReplicationMsg::Snapshot { .. });
        if snapshot_frames > 0 && !is_snapshot {
            bail!("state message interleaved with a snapshot");
        }
        let msg = match msg.validate(clock::current_micros()) {
            Ok(msg) => msg,
            Err(err) if is_snapshot => bail!("snapshot frame rejected: {err}"),
            Err(err) => {
                warn!(%peer, %err, "state message rejected");
                continue;
            }
        };
        let more = matches!(msg, Received::Snapshot { more: true, .. });

        let mut sender = Sender::new(peer, did, &link.view().identity().resolver);
        let Some(change) = prove(msg, &mut sender).await else {
            continue;
        };
        let change = match change {
            Remote::Snapshot(docs) => {
                snapshot.extend(docs);
                snapshot_frames += 1;
                if snapshot_frames > MAX_SNAPSHOT_FRAMES {
                    bail!("snapshot exceeds {MAX_SNAPSHOT_FRAMES} frames");
                }
                if more {
                    continue;
                }
                snapshot_frames = 0;
                Remote::Snapshot(std::mem::take(&mut snapshot))
            }
            change => change,
        };

        // Awaited rather than tried: a full queue holds this stream back,
        // and QUIC flow control passes that on to the peer.
        link.view()
            .commands()
            .push(move |world: &mut World| guards::apply_remote(world, peer_ent, change))
            .send()
            .await
            .context("async command queue closed")?;
    }
}

/// The peer a frame came from: its endpoint, the DID it proved, and that DID's
/// document, resolved the first time a signature needs it.
struct Sender<'a> {
    id:       EndpointId,
    did:      Option<&'a Did>,
    resolver: &'a Resolver,
    document: Option<Document>,
}

impl<'a> Sender<'a> {
    const fn new(id: EndpointId, did: Option<&'a Did>, resolver: &'a Resolver) -> Self {
        Self {
            id,
            did,
            resolver,
            document: None,
        }
    }

    /// The proven DID's document. Not resolved again for each batch.
    async fn own_document(&mut self) -> Option<&Document> {
        if self.document.is_none() {
            self.document = resolve(self.resolver, self.did?).await;
        }
        self.document.as_ref()
    }

    /// Whether a batch names this connection's endpoint and proven DID.
    fn owns(&self, batch: &Batch) -> bool {
        batch.payload.writer == self.id && self.did == Some(&batch.payload.did)
    }
}

/// `did`'s document, or `None` if it does not resolve in time.
async fn resolve(resolver: &Resolver, did: &Did) -> Option<Document> {
    match timeout(RESOLVE_TIMEOUT, resolver.resolve(did)).await {
        Ok(Ok(document)) => Some(document),
        Ok(Err(err)) => {
            warn!(%did, %err, "DID did not resolve; its session batches dropped");
            None
        }
        Err(_) => {
            warn!(%did, "DID resolution timed out; its session batches dropped");
            None
        }
    }
}

/// Checks each authorship proof and pin record against its document's
/// namespace. A pin may relay someone else's proof; each receiver records
/// whether the sender is the author.
///
/// Nothing here is gated on the sender's presence. A pin or withdrawal is
/// admitted by the record it carries, a snapshot layer by its writer's
/// signature, and a session batch from its writer's own connection, which
/// proves its endpoint and DID. Presence only drives discovery and dialing;
/// making delivery wait on it would couple one transport to another's timing.
/// Every batch's signature must also verify against the DID it names, so a bad
/// signature is refused at the first hop and no holder diverges.
async fn prove(msg: Received, sender: &mut Sender<'_>) -> Option<Remote> {
    Some(match msg {
        Received::Snapshot { docs, .. } => Remote::Snapshot(prove_docs(docs, sender).await),
        Received::Pin { author, record } => {
            let (doc, space) = (record.payload.doc, record.payload.space);
            Remote::Pin(claim(proven(doc, author)?, checked(record, doc, space)?)?)
        }
        Received::Unpin { doc, withdrawal } => Remote::Unpin {
            doc,
            withdrawal: withdrawal
                .and_then(|record| {
                    let space = record.payload.space;
                    checked(record, doc, space)
                })
                .filter(|record| !record.payload.pinned),
        },
        Received::Hold(hold) => Remote::Hold(hold),
        Received::ReleaseHold { doc } => Remote::ReleaseHold { doc },
        Received::Session(batch) => {
            if !sender.owns(&batch) {
                debug!(peer = %sender.id, "session batch not from its writer; dropped");
                return None;
            }
            let peer = sender.id;
            let document = sender.own_document().await?;
            if let Err(err) = batch.signed.verify_document(&batch.payload.did, document) {
                warn!(%peer, %err, "session batch not signed by its DID; dropped");
                return None;
            }
            Remote::Session(batch)
        }
    })
}

/// `record` if it names `doc` in `space` and `doc`'s namespace signed it.
fn checked(record: Record, doc: hsd::id::DocId, space: SpaceId) -> Option<Record> {
    match record.verify(doc, space) {
        Ok(()) => Some(record),
        Err(err) => {
            warn!(%doc, %err, "pin record dropped");
            None
        }
    }
}

/// `proof` with the DID it names, if `doc`'s namespace signed it.
fn proven(doc: hsd::id::DocId, proof: Authorship) -> Option<Proven> {
    let author = proof.verify_namespace(NamespaceId::from(&doc.0)).ok()?;
    Some(Proven { author, proof })
}

/// A pin carrying a checked record that says the document is pinned.
fn claim(proven: Proven, record: Record) -> Option<PinClaim> {
    record.payload.pinned.then_some(PinClaim { proven, record })
}

async fn prove_docs(docs: Vec<ReceivedDoc>, sender: &mut Sender<'_>) -> Vec<RemoteDoc> {
    let mut out = Vec::with_capacity(docs.len());
    for d in docs {
        let mut layers = Vec::with_capacity(d.layers.len());
        for layer in d.layers {
            layers.extend(vouched(layer, sender).await);
        }
        let pin = d.pin.and_then(|pin| {
            let record = checked(pin.record, d.doc, d.space)?;
            claim(proven(d.doc, pin.author)?, record)
        });
        let withdrawal = d
            .withdrawal
            .and_then(|record| checked(record, d.doc, d.space))
            .filter(|record| !record.payload.pinned);
        out.push(RemoteDoc {
            doc: d.doc,
            space: d.space,
            pin,
            hold: d.hold,
            withdrawal,
            layers,
        });
    }
    out
}

/// A snapshot layer's batches that the sender may state. Every batch of a
/// layer names one writer and DID. The writer's own layer needs the
/// connection's proof of that DID, and every batch needs a signature that
/// verifies against the DID's document, resolved once for the layer. Batches
/// that fail are dropped, and a DID that does not resolve drops the layer.
async fn vouched(layer: Vec<Retained>, sender: &mut Sender<'_>) -> Option<Vec<Retained>> {
    let first = &layer.first()?.batch;
    if first.payload.writer == sender.id && !sender.owns(first) {
        return None;
    }

    let did = first.payload.did.clone();
    let peer = sender.id;
    let resolved;
    let document = if sender.did == Some(&did) {
        sender.own_document().await?
    } else {
        resolved = resolve(sender.resolver, &did).await?;
        &resolved
    };
    let mut kept = Vec::with_capacity(layer.len());
    for retained in layer {
        match retained.batch.signed.verify_document(&did, document) {
            Ok(()) => kept.push(retained),
            Err(err) => warn!(%peer, %err, "session batch dropped"),
        }
    }
    (!kept.is_empty()).then_some(kept)
}

#[cfg(test)]
mod tests {
    use hsd::id::{
        DocId,
        PrimId,
    };
    use iroh_docs::NamespaceSecret;
    use unavi_identity::{
        authorship::Authorship,
        identity::Identity,
    };
    use xdid::method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    };

    use super::*;
    use crate::replication::{
        message::{
            ReceivedDoc,
            ReceivedPin,
            SessionBatch,
            SessionWrite,
        },
        record::PinRecord,
    };

    fn identity() -> Identity {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        Identity::new(did, key)
    }

    fn peer(seed: u8) -> EndpointId {
        iroh::SecretKey::from_bytes(&[seed; 32]).public()
    }

    fn prove_as(msg: Received, peer: EndpointId, did: Option<&Did>) -> Option<Remote> {
        let resolver = Resolver::new().expect("resolver");
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("runtime")
            .block_on(prove(msg, &mut Sender::new(peer, did, &resolver)))
    }

    fn namespace() -> NamespaceSecret {
        NamespaceSecret::from_bytes(&[7; 32])
    }

    fn record(key: &NamespaceSecret, doc: DocId, space: SpaceId, at: u64, pinned: bool) -> Record {
        Record::sign(
            key,
            PinRecord {
                doc,
                space,
                at,
                pinned,
            },
        )
        .expect("sign")
    }

    fn pin(author: &Identity, space: SpaceId) -> Received {
        let ns = namespace();
        let doc = DocId(*ns.id().as_bytes());
        Received::Pin {
            author: Authorship::sign(&ns, author).expect("sign"),
            record: record(&ns, doc, space, 1, true),
        }
    }

    /// A batch from `writer` naming `named`, signed by `signer`.
    fn batch(writer: EndpointId, named: &Identity, signer: &Identity, space: SpaceId) -> Batch {
        Batch::sign(
            SessionBatch {
                writer,
                did: named.did().clone(),
                doc: DocId([4; 32]),
                space,
                at: 1,
                writes: vec![SessionWrite {
                    prim:  PrimId([3; 16]),
                    name:  "state/k".to_owned(),
                    value: Some(vec![1]),
                }],
                clears: Vec::new(),
            },
            signer.signing_key(),
        )
        .expect("sign")
    }

    fn snapshot_of(batch: Batch, space: SpaceId) -> Received {
        Received::Snapshot {
            docs: vec![ReceivedDoc {
                doc: batch.payload.doc,
                space,
                pin: None,
                hold: None,
                withdrawal: None,
                layers: vec![vec![Retained {
                    batch,
                    live: vec![0],
                    live_clears: Vec::new(),
                }]],
            }],
            more: false,
        }
    }

    fn relayed_layers(remote: Option<Remote>) -> usize {
        let Some(Remote::Snapshot(docs)) = remote else {
            panic!("a snapshot is accepted");
        };
        docs.iter().map(|doc| doc.layers.len()).sum()
    }

    #[test]
    fn a_pin_relaying_the_authors_proof_is_accepted() {
        let alice = identity();
        let space = SpaceId([1; 32]);

        let Some(Remote::Pin(claim)) = prove_as(pin(&alice, space), peer(1), None) else {
            panic!("a proof that verifies is accepted whoever sends it");
        };
        assert_eq!(&claim.proven.author, alice.did());
    }

    #[test]
    fn a_proof_for_another_namespace_is_dropped() {
        let alice = identity();
        let space = SpaceId([1; 32]);
        let Received::Pin { author, .. } = pin(&alice, space) else {
            unreachable!();
        };
        let other = NamespaceSecret::from_bytes(&[9; 32]);
        let other_doc = DocId(*other.id().as_bytes());

        let msg = Received::Pin {
            author,
            record: record(&other, other_doc, space, 1, true),
        };
        assert!(prove_as(msg, peer(1), None).is_none());
    }

    #[test]
    fn a_forged_pin_record_drops_the_pin() {
        let alice = identity();
        let space = SpaceId([1; 32]);
        let ns = namespace();
        let doc = DocId(*ns.id().as_bytes());
        let other = NamespaceSecret::from_bytes(&[8; 32]);
        let Received::Pin { author, .. } = pin(&alice, space) else {
            unreachable!();
        };
        let with = |record: Record| Received::Pin {
            author: author.clone(),
            record,
        };

        assert!(
            prove_as(with(record(&ns, doc, space, 2, true)), peer(1), None).is_some(),
            "a genuine record passes"
        );
        let mut wrong_key = record(&other, DocId(*other.id().as_bytes()), space, 2, true);
        wrong_key.payload.doc = doc;
        assert!(
            prove_as(with(wrong_key), peer(1), None).is_none(),
            "a record another namespace's key signed"
        );
        assert!(
            prove_as(
                with(record(&ns, DocId([3; 32]), space, 2, true)),
                peer(1),
                None
            )
            .is_none(),
            "a record the namespace signed for another document id"
        );
        assert!(
            prove_as(with(record(&ns, doc, space, 2, false)), peer(1), None).is_none(),
            "a pin must carry a record that says pinned"
        );
    }

    #[test]
    fn a_withdrawal_is_kept_only_when_genuine() {
        let space = SpaceId([1; 32]);
        let ns = namespace();
        let doc = DocId(*ns.id().as_bytes());
        let withdrawn = |record: Record| {
            let msg = Received::Unpin {
                doc,
                withdrawal: Some(record),
            };
            let Some(Remote::Unpin { withdrawal, .. }) = prove_as(msg, peer(1), None) else {
                panic!("an unpin is never dropped for its record");
            };
            withdrawal.is_some()
        };

        assert!(withdrawn(record(&ns, doc, space, 2, false)));
        assert!(
            withdrawn(record(&ns, doc, SpaceId([2; 32]), 2, false)),
            "a genuine record is admitted whatever space it names"
        );
        assert!(
            !withdrawn(record(&ns, doc, space, 2, true)),
            "a pin is not a withdrawal"
        );
        let other = NamespaceSecret::from_bytes(&[8; 32]);
        let mut forged = record(&other, DocId(*other.id().as_bytes()), space, 2, false);
        forged.payload.doc = doc;
        assert!(!withdrawn(forged), "signed by another namespace");
    }

    #[test]
    fn a_snapshot_pin_needs_a_genuine_pinned_record_and_a_withdrawal_a_genuine_withdrawal() {
        let alice = identity();
        let space = SpaceId([1; 32]);
        let ns = namespace();
        let doc = DocId(*ns.id().as_bytes());
        let proof = Authorship::sign(&ns, &alice).expect("sign");
        let snapshot = |pin: Option<Record>, withdrawal: Option<Record>| {
            let msg = Received::Snapshot {
                docs: vec![ReceivedDoc {
                    doc,
                    space,
                    pin: pin.map(|record| ReceivedPin {
                        author: proof.clone(),
                        record,
                    }),
                    hold: None,
                    withdrawal,
                    layers: Vec::new(),
                }],
                more: false,
            };
            let Some(Remote::Snapshot(mut docs)) = prove_as(msg, peer(1), None) else {
                panic!("a snapshot is accepted");
            };
            docs.remove(0)
        };

        let genuine = snapshot(Some(record(&ns, doc, space, 2, true)), None);
        assert!(genuine.pin.is_some() && genuine.withdrawal.is_none());
        let both = snapshot(
            Some(record(&ns, doc, space, 2, true)),
            Some(record(&ns, doc, space, 3, false)),
        );
        assert!(
            both.pin.is_some() && both.withdrawal.is_some(),
            "a pin is stated whatever withdrawal the sender also knows"
        );
        let stale = snapshot(Some(record(&ns, doc, space, 3, false)), None);
        assert!(stale.pin.is_none(), "a pin record must say pinned");
        let wrong_space = snapshot(Some(record(&ns, doc, SpaceId([2; 32]), 2, true)), None);
        assert!(wrong_space.pin.is_none());
        let alone = snapshot(None, Some(record(&ns, doc, space, 3, false)));
        assert!(alone.pin.is_none() && alone.withdrawal.is_some());
        let not_a_withdrawal = snapshot(None, Some(record(&ns, doc, space, 3, true)));
        assert!(not_a_withdrawal.withdrawal.is_none());
        let forged = snapshot(None, Some(record(&ns, doc, SpaceId([2; 32]), 3, false)));
        assert!(forged.withdrawal.is_none());
    }

    #[test]
    fn a_pin_is_not_gated_on_the_senders_presence() {
        let alice = identity();
        assert!(
            prove_as(pin(&alice, SpaceId([2; 32])), peer(1), None).is_some(),
            "a genuine pin record is admitted whatever space it names"
        );
    }

    #[test]
    fn a_live_batch_is_accepted_only_from_its_writer_proving_its_did() {
        let (alice, mallory) = (identity(), identity());
        let space = SpaceId([1; 32]);
        let own = || Received::Session(batch(peer(1), &alice, &alice, space));

        assert!(prove_as(own(), peer(1), Some(alice.did())).is_some());
        assert!(
            prove_as(own(), peer(1), None).is_none(),
            "a sender with no proven DID writes no session state"
        );
        assert!(
            prove_as(own(), peer(1), Some(mallory.did())).is_none(),
            "the batch names a DID the connection did not prove"
        );
        assert!(
            prove_as(own(), peer(2), Some(alice.did())).is_none(),
            "the batch names another endpoint's layer"
        );
    }

    #[test]
    fn a_snapshot_keeps_its_layers_and_occupancy() {
        let alice = identity();
        let space = SpaceId([1; 32]);
        let msg = Received::Snapshot {
            docs: vec![ReceivedDoc {
                doc: DocId([4; 32]),
                space,
                pin: None,
                hold: Some(1),
                withdrawal: None,
                layers: vec![vec![Retained {
                    batch:       batch(peer(1), &alice, &alice, space),
                    live:        vec![0],
                    live_clears: Vec::new(),
                }]],
            }],
            more: false,
        };

        let Some(Remote::Snapshot(docs)) = prove_as(msg, peer(1), Some(alice.did())) else {
            panic!("a snapshot is accepted");
        };
        assert_eq!(docs[0].layers.len(), 1);
        assert_eq!(docs[0].hold, Some(1), "occupancy is not gated on presence");
    }

    #[test]
    fn a_writers_own_batch_with_a_bad_signature_is_refused_at_the_first_hop() {
        let (alice, mallory) = (identity(), identity());
        let space = SpaceId([1; 32]);
        let forged = || batch(peer(1), &alice, &mallory, space);

        let live = prove_as(Received::Session(forged()), peer(1), Some(alice.did()));
        assert!(
            live.is_none(),
            "the connection proves the DID, not the signature"
        );
        let snapshot = prove_as(snapshot_of(forged(), space), peer(1), Some(alice.did()));
        assert_eq!(relayed_layers(snapshot), 0);
    }

    #[test]
    fn a_relayed_batch_is_kept_only_when_its_signature_verifies() {
        let (alice, mallory) = (identity(), identity());
        let space = SpaceId([1; 32]);
        let carrier = peer(2);

        let genuine = batch(peer(1), &alice, &alice, space);
        let relayed = prove_as(snapshot_of(genuine, space), carrier, None);
        assert_eq!(relayed_layers(relayed), 1, "a signed batch is relayable");

        let forged = batch(peer(1), &alice, &mallory, space);
        let relayed = prove_as(snapshot_of(forged, space), carrier, None);
        assert_eq!(
            relayed_layers(relayed),
            0,
            "a batch naming a DID it was not signed by is dropped"
        );
    }

    #[test]
    fn a_writers_own_layer_in_a_snapshot_needs_its_proven_did() {
        let (alice, mallory) = (identity(), identity());
        let space = SpaceId([1; 32]);
        let own = || snapshot_of(batch(peer(1), &alice, &alice, space), space);

        let accepted = prove_as(own(), peer(1), Some(alice.did()));
        assert_eq!(relayed_layers(accepted), 1);
        let refused = prove_as(own(), peer(1), Some(mallory.did()));
        assert_eq!(
            relayed_layers(refused),
            0,
            "the writer's own layer is not checked by signature, so the connection must vouch"
        );
    }
}
