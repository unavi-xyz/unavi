use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Full player state for initial sync over a direct peer stream.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayerStateMsg {
    pub objects: Vec<ObjectStateEntry>,
    pub portals: Vec<PortalStateEntry>,
}

/// A spawned HSD object owned by a player.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectStateEntry {
    pub record_id: [u8; 32],
    pub node_id: SmolStr,
}

/// A portal in a player's state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortalStateEntry {
    pub transform: WireTransform,
    pub size: [f32; 2],
    pub dest_space: [u8; 32],
    pub dest_transform: WireTransform,
    pub dest_size: [f32; 2],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

/// Incremental state change — broadcast on gossip to all peers in a shared space.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateDeltaMsg {
    pub sender: iroh::EndpointId,
    pub op: StateDeltaOp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateDeltaOp {
    ObjectAdded(ObjectStateEntry),
    ObjectRemoved {
        record_id: [u8; 32],
        node_id: SmolStr,
    },
    PortalAdded(PortalStateEntry),
    PortalRemoved {
        dest_space: [u8; 32],
    },
}

/// Sent by requester to trigger a full `PlayerStateMsg` response.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StateRequestMsg;
