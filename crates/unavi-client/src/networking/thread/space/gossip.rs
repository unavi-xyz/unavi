//! Gossip message types for space protocol.

use iroh::{EndpointAddr, EndpointId};
use serde::{Deserialize, Serialize};
use wds::signed_bytes::Signable;

use super::types::object_id::ObjectId;

/// Timestamp for ownership ordering (milliseconds since Unix epoch).
pub type OwnershipTimestamp = u64;

/// Broadcast to a space gossip topic that you are joining.
///
/// Message is signed and verified with the sender's endpoint ID.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinBroadcast {
    pub endpoint: EndpointAddr,
}

impl Signable for JoinBroadcast {}

/// Broadcast to claim ownership of a dynamic object.
///
/// Signed with the claimer's endpoint secret key. Receivers verify the
/// signature matches the claimed endpoint ID.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectClaimBroadcast {
    /// The object being claimed.
    pub object_id: ObjectId,
    /// Claimer's endpoint ID (verified via signature).
    pub claimer: EndpointId,
    /// Timestamp when claim was made (for conflict resolution).
    pub timestamp: OwnershipTimestamp,
    /// Sequence number for same-timestamp tiebreaking.
    pub seq: u32,
}

impl Signable for ObjectClaimBroadcast {}

/// Broadcast to release ownership of a dynamic object.
///
/// Only valid if the releaser is the current owner.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectReleaseBroadcast {
    /// The object being released.
    pub object_id: ObjectId,
    /// Releaser's endpoint ID (must match current owner).
    pub releaser: EndpointId,
    /// Timestamp of release.
    pub timestamp: OwnershipTimestamp,
}

impl Signable for ObjectReleaseBroadcast {}

/// Unified gossip message envelope for space protocol.
///
/// All messages are signed with [`SignedBytes`] before broadcast.
///
/// [`SignedBytes`]: wds::signed_bytes::SignedBytes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpaceGossipMsg {
    Join(JoinBroadcast),
    ObjectClaim(ObjectClaimBroadcast),
    ObjectRelease(ObjectReleaseBroadcast),
}

impl Signable for SpaceGossipMsg {}

impl SpaceGossipMsg {
    /// Get the endpoint ID that should have signed this message.
    pub const fn signer_id(&self) -> EndpointId {
        match self {
            Self::Join(m) => m.endpoint.id,
            Self::ObjectClaim(m) => m.claimer,
            Self::ObjectRelease(m) => m.releaser,
        }
    }
}
