use bevy::prelude::*;
use iroh::EndpointId;
use unavi_policy::{
    registry::Policy,
    trust::{
        self,
        Trust,
    },
};
use unavi_store::local::Storage;

use crate::{
    connection::disconnect,
    state::replicas,
};

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
pub fn eject(policy: &Policy, peer: EndpointId, storage: &Storage) -> Result<(), NoIdentity> {
    set_rung(policy, peer, Some(Trust::Blocked), storage)?;

    let reverted = replicas::revert_writes(peer);
    info!(reverted, "Ejected peer");

    disconnect(peer);
    Ok(())
}

/// Lifts a block, so the peer is judged by the default again.
pub fn unblock(policy: &Policy, peer: EndpointId, storage: &Storage) -> Result<(), NoIdentity> {
    set_rung(policy, peer, None, storage)
}

/// Marks `peer` as one the local user trusts, raising what its content may
/// consume.
pub fn trust_peer(policy: &Policy, peer: EndpointId, storage: &Storage) -> Result<(), NoIdentity> {
    set_rung(policy, peer, Some(Trust::Trusted), storage)
}

/// Records `rung` for `peer`, or clears it when `rung` is `None`.
///
/// The peer's quota is dropped rather than adjusted, so the next document it
/// owns derives its caps from the new rung. Adjusting in place would have to
/// re-scale buckets that are partly spent.
fn set_rung(
    policy: &Policy,
    peer: EndpointId,
    rung: Option<Trust>,
    storage: &Storage,
) -> Result<(), NoIdentity> {
    let did = crate::identity::bindings()
        .and_then(|b| b.did_of(peer))
        .ok_or(NoIdentity)?;

    match rung {
        Some(rung) => trust::set_override(did, rung),
        None => trust::clear_override(&did),
    }
    policy.forget_peer(peer);

    if let Err(err) = trust::save(storage) {
        warn!(?err, "failed to persist the trust table");
    }
    Ok(())
}

/// A peer that proved no DID, and so has nothing durable to block.
#[derive(Debug, thiserror::Error)]
#[error("peer proved no identity to block")]
pub struct NoIdentity;
