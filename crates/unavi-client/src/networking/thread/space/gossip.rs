use iroh::{EndpointAddr, EndpointId};
use serde::{Deserialize, Serialize};
use wds::signed_bytes::Signable;

use super::types::state::StateDeltaMsg;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpaceGossip {
    pub sender: EndpointId,
    pub msg: SpaceGossipMsg,
}

impl Signable for SpaceGossip {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpaceGossipMsg {
    Join(EndpointAddr),
    StateDelta(StateDeltaMsg),
}

// TODO move object claiming to peer state
