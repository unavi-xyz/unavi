use std::path::Path;

use bevy::prelude::*;
use hsd::id::DocId;
use iroh::EndpointId;
use iroh_docs::NamespaceId;
use unavi_policy::{
    check::{
        self,
        Resolver,
    },
    identity,
    trust::{
        self,
        Trust,
    },
};
use unavi_util::async_task::spawn_async_task;
use xdid::core::did::Did;

use crate::{
    connection::disconnect,
    peer::self_peer_id,
    state::replicas,
};

/// Teaches policy how to attribute a document, without policy depending on the
/// networking crate to do it.
pub fn install_resolver() {
    fn owner(space: DocId, doc: DocId) -> Option<[u8; 32]> {
        replicas::owner(NamespaceId::from(&space.0), NamespaceId::from(&doc.0))
    }
    fn space_of(doc: DocId) -> Option<DocId> {
        replicas::space_of(NamespaceId::from(&doc.0)).map(|ns| DocId(*ns.as_bytes()))
    }
    check::set_resolver(Resolver {
        owner,
        space_of,
        self_peer: self_peer_id,
    });
}

/// Where the trust table is kept, or `None` on wasm, which has no filesystem
/// and so holds a block only for the life of the tab.
#[cfg(target_family = "wasm")]
const fn table_dir() -> Option<&'static Path> {
    None
}

#[cfg(not(target_family = "wasm"))]
#[expect(
    clippy::unnecessary_wraps,
    reason = "mirrors the wasm variant, which returns None"
)]
fn table_dir() -> Option<&'static Path> {
    Some(unavi_util::dirs::data_local_dir())
}

/// Loads the local trust table, which is the user's own opinion and so has to
/// outlive the session.
pub fn load_trust_table() {
    let Some(dir) = table_dir() else {
        return;
    };
    if let Err(err) = trust::load(dir) {
        error!(
            ?err,
            "Trust table could not be read; every block is inactive this session"
        );
    }
}

/// Blocks `peer` and undoes what they contributed, in one call.
///
/// Retroactive by construction. Their pins, authority claims and owner-authored
/// KV hang off the `RemotePeer` entity and cascade away when the connection
/// drops; their neutral cells are rolled back explicitly, since those
/// deliberately outlive a disconnect. The rung is set first so a reconnect
/// while the rest is still unwinding does not arrive as a guest.
///
/// A peer that proved no DID can still be disconnected but not durably blocked:
/// there is no stable identity to record the decision against, and an endpoint
/// id is not one.
pub fn eject(peer: [u8; 32]) -> Result<(), NoIdentity> {
    let did = identity::did_of(peer).ok_or(NoIdentity)?;
    trust::set_override(did, Trust::Blocked);

    let reverted = replicas::revert_neutral_writes(peer);
    unavi_quota::registry::forget_peer(NamespaceId::from(&peer));
    info!(reverted, "Ejected peer");

    if let Some(dir) = table_dir()
        && let Err(err) = trust::save(dir)
    {
        warn!(?err, "failed to persist the block");
    }

    if let Ok(endpoint) = EndpointId::from_bytes(&peer) {
        disconnect(endpoint);
    }
    Ok(())
}

/// Lifts a block, so the peer is judged by the graph or the default again.
pub fn unblock(peer: [u8; 32]) -> Result<(), NoIdentity> {
    let did = identity::did_of(peer).ok_or(NoIdentity)?;
    trust::clear_override(&did);
    unavi_quota::registry::forget_peer(NamespaceId::from(&peer));
    if let Some(dir) = table_dir()
        && let Err(err) = trust::save(dir)
    {
        warn!(?err, "failed to persist the unblock");
    }
    Ok(())
}

/// Drops the connection to a peer whose proven DID turns out to be blocked.
///
/// The gate cannot sit at accept time: the binding is established over the
/// connection itself, so at accept there is no DID to judge and every peer
/// reads as an unproven guest. This is the first moment a block can be applied
/// to an incoming peer at all.
pub fn enforce_block(peer: EndpointId, did: &Did) -> bool {
    if trust::of_did(did) != Trust::Blocked {
        return false;
    }
    info!(%did, "Refusing a blocked peer");
    disconnect(peer);
    true
}

/// A peer that proved no DID, and so has nothing durable to block.
#[derive(Debug, thiserror::Error)]
#[error("peer proved no identity to block")]
pub struct NoIdentity;

/// Where a vouch lives in the voucher's root doc.
const VOUCH_PREFIX: &str = "vouches/";

/// Subjects the last publish put on the wire, so a retracted vouch can be
/// tombstoned rather than left readable forever.
static PUBLISHED: parking_lot::Mutex<Option<std::collections::HashSet<[u8; 32]>>> =
    parking_lot::Mutex::new(None);

/// Publishes the local vouch list under salted hashes, and empties the keys of
/// any vouch since retracted.
///
/// Only the hashes go out: anyone may test whether a peer they can already
/// name is vouched for, and nobody can enumerate the list.
pub fn publish_vouches(docs: Query<&bevy_wds::LocalDocs>) {
    use std::collections::HashSet;

    use bytes::Bytes;
    use unavi_policy::trust::vouch::{
        Vouch,
        subject_hash,
    };

    let Some(ns) = bevy_wds::root_doc() else {
        return;
    };
    let Ok(docs) = docs.single().map(|d| d.0.clone()) else {
        return;
    };

    let salt = trust::salt();
    let vouches = trust::my_vouches()
        .into_iter()
        .map(|(did, weight)| Vouch {
            subject: subject_hash(&salt, &did),
            weight,
        })
        .collect::<Vec<_>>();

    let subjects = vouches.iter().map(|v| v.subject).collect::<HashSet<_>>();
    let retracted = PUBLISHED
        .lock()
        .replace(subjects.clone())
        .map(|last| last.difference(&subjects).copied().collect::<Vec<_>>())
        .unwrap_or_default();

    if vouches.is_empty() && retracted.is_empty() {
        return;
    }

    spawn_async_task(async move {
        for vouch in vouches {
            let Ok(bytes) = postcard::to_allocvec(&vouch) else {
                continue;
            };
            let key = format!("{VOUCH_PREFIX}{}", hex(&vouch.subject));
            if let Err(err) = wds::kv::set(&docs, ns, &key, Bytes::from(bytes)).await {
                warn!(?err, "failed to publish vouch");
            }
        }
        for subject in retracted {
            let key = format!("{VOUCH_PREFIX}{}", hex(&subject));
            if let Err(err) = wds::kv::set(&docs, ns, &key, Bytes::new()).await {
                warn!(?err, "failed to retract vouch");
            }
        }
    });
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}
