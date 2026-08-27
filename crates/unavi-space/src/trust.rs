use std::sync::Arc;

use bevy::prelude::*;
use bevy_iroh::store::LocalStore;
use hsd::id::DocId;
use iroh::EndpointId;
use iroh_docs::NamespaceId;
use unavi_identity::auth::bindings::Bindings;
use unavi_policy::{
    check::{
        self,
        Resolver,
    },
    trust::{
        self,
        Trust,
    },
};
use unavi_store::local::Storage;
use unavi_util::async_task::spawn_async_task;

use crate::{
    connection::disconnect,
    peer::self_peer_id,
    state::replicas,
};

/// Registers with policy how to resolve a document's owning space and peer,
/// so attribution needs no dependency on this crate.
pub fn install_resolver(bindings: Arc<Bindings>) {
    fn owner(space: DocId, doc: DocId) -> Option<[u8; 32]> {
        replicas::owner(NamespaceId::from(&space.0), NamespaceId::from(&doc.0))
    }
    fn space_of(doc: DocId) -> Option<DocId> {
        replicas::space_of(NamespaceId::from(&doc.0)).map(|ns| DocId(*ns.as_bytes()))
    }
    check::set_resolver(Resolver {
        bindings,
        owner,
        space_of,
        self_peer: self_peer_id,
    });
}

/// Where the trust table persists, for the reads and writes that happen after
/// the plugin has built.
#[derive(Resource, Clone)]
pub struct TrustStorage(pub Storage);

/// Loads the persisted local trust table.
pub fn load_trust_table(storage: &Storage) {
    if let Err(err) = trust::load(storage) {
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
pub fn eject(peer: [u8; 32], storage: &Storage) -> Result<(), NoIdentity> {
    let did = crate::identity::bindings()
        .and_then(|b| b.did_of_bytes(&peer))
        .ok_or(NoIdentity)?;
    trust::set_override(did, Trust::Blocked);

    let reverted = replicas::revert_neutral_writes(peer);
    unavi_quota::registry::forget_peer(NamespaceId::from(&peer));
    info!(reverted, "Ejected peer");

    if let Err(err) = trust::save(storage) {
        warn!(?err, "failed to persist the block");
    }

    if let Ok(endpoint) = EndpointId::from_bytes(&peer) {
        disconnect(endpoint);
    }
    Ok(())
}

/// Lifts a block, so the peer is judged by the graph or the default again.
pub fn unblock(peer: [u8; 32], storage: &Storage) -> Result<(), NoIdentity> {
    let did = crate::identity::bindings()
        .and_then(|b| b.did_of_bytes(&peer))
        .ok_or(NoIdentity)?;
    trust::clear_override(&did);
    unavi_quota::registry::forget_peer(NamespaceId::from(&peer));
    if let Err(err) = trust::save(storage) {
        warn!(?err, "failed to persist the unblock");
    }
    Ok(())
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
pub fn publish_vouches(stores: Query<&LocalStore>) {
    use std::collections::HashSet;

    use unavi_policy::trust::vouch::{
        Vouch,
        subject_hash,
    };

    let Ok(store) = stores.single().map(|s| s.0.clone()) else {
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
        let published = async {
            let root = store.open(store.root()).await?;
            for vouch in vouches {
                let key = format!(
                    "{VOUCH_PREFIX}{}",
                    unavi_store::local::encode_hex(&vouch.subject)
                );
                root.set(key, postcard::to_allocvec(&vouch)?).await?;
            }
            for subject in retracted {
                root.remove(format!(
                    "{VOUCH_PREFIX}{}",
                    unavi_store::local::encode_hex(&subject)
                ))
                .await?;
            }
            anyhow::Ok(())
        }
        .await;

        if let Err(err) = published {
            warn!(?err, "failed to publish vouches");
        }
    });
}
