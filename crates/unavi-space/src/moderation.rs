//! The local user's verdicts on peers: eject, block, unblock, trust.

use bevy::prelude::*;
use iroh::EndpointId;
use unavi_policy::{
    ledger::PeerKey,
    quota::limits::Limits,
    trust::Trust,
};

use crate::{
    authority::SpaceView,
    link::PeerLink,
    replication::guards,
};

/// Blocks `peer` and undoes what they contributed.
///
/// The block is written before anything unwinds, so a reconnect arriving
/// mid-teardown is not readmitted. A peer that proved a DID is blocked by it,
/// durably; one that proved none is blocked by endpoint for this session.
/// Pins and holds cascade away with the connection; session cells outlive a
/// disconnect, so they are rolled back by hand.
pub fn eject(view: &SpaceView, link: &PeerLink, peer: EndpointId) {
    if set_trust(view, peer, Some(Trust::Blocked)).is_err() {
        view.trust().block_endpoint(peer);
    }

    let reverted = guards::revert_session(view.replicas(), view.commands(), peer);
    info!(reverted, "Ejected peer");

    link.disconnect(peer);
}

/// Lifts a block, so the peer is judged by the default again.
pub fn unblock(view: &SpaceView, peer: EndpointId) -> Result<(), NoIdentity> {
    view.trust().unblock_endpoint(peer);
    set_trust(view, peer, None)
}

/// Marks `peer` as one the local user trusts, raising what its content may
/// consume.
pub fn trust_peer(view: &SpaceView, peer: EndpointId) -> Result<(), NoIdentity> {
    set_trust(view, peer, Some(Trust::Trusted))
}

/// Records `trust` for `peer`, or clears it when `trust` is `None`.
///
/// The peer's quota takes the new level's limits in place, so documents it
/// already owns are bound by them at once.
fn set_trust(view: &SpaceView, peer: EndpointId, trust: Option<Trust>) -> Result<(), NoIdentity> {
    let did = view.identity().bindings.did_of(peer).ok_or(NoIdentity)?;

    match trust {
        Some(trust) => {
            if let Err(err) = view.trust().set(did.clone(), trust) {
                warn!(%err, "refused trust level");
                return Ok(());
            }
        }
        None => view.trust().clear(&did),
    }
    let limits = Limits::for_trust(view.trust_of_did(&did));
    view.policy().retrust_peer(&PeerKey::Did(did), limits);

    if let Err(err) = view.trust().save() {
        warn!(?err, "failed to persist the trust table");
    }
    Ok(())
}

/// A peer that proved no DID, and so has nothing durable to block.
#[derive(Debug, thiserror::Error)]
#[error("peer proved no identity to block")]
pub struct NoIdentity;
