use bevy::prelude::*;
use iroh::EndpointId;
use unavi_policy::trust::Trust;

use crate::{
    connection::PeerLink,
    view::SpaceView,
};

/// Blocks `peer` and undoes what they contributed.
///
/// The rung is written before anything unwinds, so a reconnect arriving mid-
/// teardown is not readmitted as a guest. Pins, authority claims and
/// owner-authored KV cascade away with the connection; only neutral cells need
/// rolling back by hand, since they outlive a disconnect.
pub fn eject(view: &SpaceView, link: &PeerLink, peer: EndpointId) -> Result<(), NoIdentity> {
    set_rung(view, peer, Some(Trust::Blocked))?;

    let reverted = view.replicas().revert_writes(peer);
    info!(reverted, "Ejected peer");

    link.disconnect(peer);
    Ok(())
}

/// Lifts a block, so the peer is judged by the default again.
pub fn unblock(view: &SpaceView, peer: EndpointId) -> Result<(), NoIdentity> {
    set_rung(view, peer, None)
}

/// Marks `peer` as one the local user trusts, raising what its content may
/// consume.
pub fn trust_peer(view: &SpaceView, peer: EndpointId) -> Result<(), NoIdentity> {
    set_rung(view, peer, Some(Trust::Trusted))
}

/// Records `rung` for `peer`, or clears it when `rung` is `None`.
///
/// The peer's quota is dropped rather than adjusted, so the next document it
/// owns derives its caps from the new rung. Adjusting in place would have to
/// re-scale buckets that are partly spent.
fn set_rung(view: &SpaceView, peer: EndpointId, rung: Option<Trust>) -> Result<(), NoIdentity> {
    let did = view.identity().bindings.did_of(peer).ok_or(NoIdentity)?;

    match rung {
        Some(rung) => view.trust().set(did, rung),
        None => view.trust().clear(&did),
    }
    view.policy().forget_peer(peer);

    if let Err(err) = view.trust().save() {
        warn!(?err, "failed to persist the trust table");
    }
    Ok(())
}

/// A peer that proved no DID, and so has nothing durable to block.
#[derive(Debug, thiserror::Error)]
#[error("peer proved no identity to block")]
pub struct NoIdentity;
