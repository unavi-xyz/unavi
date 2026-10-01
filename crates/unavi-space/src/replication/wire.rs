//! The state stream: length-prefixed [`ReplicationMsg`] frames, checked and
//! proven before they reach the world.

use std::{
    collections::HashSet,
    time::Duration,
};

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
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use tracing::warn;
use xdid::core::did::Did;

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
        guards::{
            self,
            Proven,
            Remote,
            RemoteDoc,
        },
        message::ReplicationMsg,
    },
};

/// Largest frame either side sends or accepts.
const MAX_STATE_FRAME: usize = 256 * 1024;

/// Bytes allocated ahead of a frame's content; the rest grows as it arrives,
/// so a length prefix alone never costs a full frame.
const READ_CHUNK: usize = 16 * 1024;

/// How long a message naming a space waits for its sender's presence there.
const PRESENCE_WAIT: Duration = Duration::from_secs(10);

pub async fn send_state_stream(link: &PeerLink, connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
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
            write_frame(&mut tx, &msg).await?;
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
    let claim_did = did.clone();
    let peer_ent = link
        .view()
        .commands()
        .send_with(move |world: &mut World| {
            guards::claim_remote_peer(world, peer, claim_did, generation)
        })
        .await
        .context("claim remote peer")?;

    let res = recv_loop(link, peer_ent, peer, did.as_ref(), &mut rx).await;
    let _ = link
        .view()
        .commands()
        .push(move |world: &mut World| {
            guards::release_remote_peer(world, peer_ent, generation);
        })
        .send()
        .await;
    res
}

async fn recv_loop(
    link: &PeerLink,
    peer_ent: Entity,
    peer: EndpointId,
    did: Option<&Did>,
    rx: &mut RecvStream,
) -> anyhow::Result<()> {
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
        let msg = match msg.validate(clock::current_micros()) {
            Ok(msg) => msg,
            Err(err) => {
                warn!(%peer, %err, "state message rejected");
                continue;
            }
        };

        let mut present = HashSet::new();
        for space in msg.spaces() {
            if present.contains(&space) {
                continue;
            }
            if link.presence().wait(peer, space, PRESENCE_WAIT).await {
                present.insert(space);
            } else {
                debug!(%peer, %space, "state for a space the peer is not in; dropped");
            }
        }

        let Some(change) = prove(msg, did, &present) else {
            continue;
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

/// Checks each authorship proof against the DID the sender proved, and drops
/// whatever names a space the sender is not present in.
fn prove(msg: ReplicationMsg, did: Option<&Did>, present: &HashSet<SpaceId>) -> Option<Remote> {
    let proven = |doc: hsd::id::DocId, proof: unavi_identity::authorship::Authorship| {
        let author = proof.verify_namespace(NamespaceId::from(&doc.0)).ok()?;
        if did != Some(&author) {
            debug!(%doc, "pin from a peer that is not the document's author; dropped");
            return None;
        }
        Some(Proven { author, proof })
    };

    Some(match msg {
        ReplicationMsg::Snapshot(docs) => Remote::Snapshot(
            docs.into_iter()
                .filter(|d| present.contains(&d.space))
                .map(|d| RemoteDoc {
                    doc:     d.doc,
                    space:   d.space,
                    pin:     d.pin.and_then(|proof| proven(d.doc, proof)),
                    hold:    d.hold,
                    session: d.session,
                })
                .collect(),
        ),
        ReplicationMsg::Pin { doc, space, author } => {
            if !present.contains(&space) {
                return None;
            }
            Remote::Pin {
                doc,
                space,
                proof: proven(doc, author)?,
            }
        }
        ReplicationMsg::Unpin { doc } => Remote::Unpin { doc },
        ReplicationMsg::Hold { doc, space } => {
            if !present.contains(&space) {
                return None;
            }
            Remote::Hold { doc, space }
        }
        ReplicationMsg::ReleaseHold { doc, to } => Remote::ReleaseHold { doc, to },
        ReplicationMsg::Session {
            doc,
            space,
            writes,
            at,
        } => {
            if !present.contains(&space) {
                return None;
            }
            Remote::Session {
                doc,
                space,
                writes,
                at,
            }
        }
    })
}

#[cfg(test)]
mod tests {
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

    fn identity() -> Identity {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        Identity::new(did, key)
    }

    fn pin(author: &Identity, space: SpaceId) -> ReplicationMsg {
        let ns = NamespaceSecret::from_bytes(&[7; 32]);
        ReplicationMsg::Pin {
            doc: hsd::id::DocId(*ns.id().as_bytes()),
            space,
            author: Authorship::sign(&ns, author).expect("sign"),
        }
    }

    #[test]
    fn only_the_proven_author_may_pin() {
        let (alice, mallory) = (identity(), identity());
        let space = SpaceId([1; 32]);
        let present = HashSet::from([space]);

        assert!(prove(pin(&alice, space), Some(alice.did()), &present).is_some());
        assert!(
            prove(pin(&alice, space), Some(mallory.did()), &present).is_none(),
            "replaying another DID's proof does not make the sender its author"
        );
        assert!(prove(pin(&alice, space), None, &present).is_none());
    }

    #[test]
    fn a_pin_for_a_space_the_sender_is_not_in_is_dropped() {
        let alice = identity();
        let present = HashSet::from([SpaceId([1; 32])]);
        assert!(prove(pin(&alice, SpaceId([2; 32])), Some(alice.did()), &present).is_none());
    }
}
