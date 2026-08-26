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

/// Registers with policy how to resolve a document's owning space and peer,
/// so attribution needs no dependency on this crate.
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

/// Where the trust table persists, or `None` on wasm, which has no filesystem
/// and holds blocks only for the life of the tab.
#[cfg_attr(
    target_family = "wasm",
    expect(clippy::missing_const_for_fn, reason = "the wasm arm is a constant")
)]
#[cfg_attr(
    not(target_family = "wasm"),
    expect(
        clippy::unnecessary_wraps,
        reason = "mirrors the wasm arm, which returns None"
    )
)]
fn table_dir() -> Option<&'static Path> {
    cfg_select! {
        target_family = "wasm" => None,
        _ => Some(unavi_util::dirs::data_local_dir()),
    }
}

/// Loads the persisted local trust table.
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

/// Blocks `peer` and undoes what they contributed.
///
/// The rung is written before anything unwinds, so a reconnect arriving mid-
/// teardown is not readmitted as a guest. Pins, authority claims and
/// owner-authored KV cascade away with the connection; only neutral cells need
/// rolling back by hand, since they outlive a disconnect.
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
/// The earliest a block can hit an incoming peer: the DID binding is
/// established over the connection itself, so at accept time there is nothing
/// to judge.
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

/// Publishes the local vouch list under salted subject hashes, emptying the
/// keys of any vouch since retracted. Only hashes go out, so the list cannot
/// be enumerated.
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
