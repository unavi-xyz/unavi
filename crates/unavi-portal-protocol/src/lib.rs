use serde::{
    Deserialize,
    Serialize,
};

pub const INCOMING_CHANNEL: &str = "unavi:portal:incoming";
pub const BACKLINK_CHANNEL: &str = "unavi:portal:backlink";

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct LinkState {
    pub target_space:  [u8; 32],
    pub receptor_doc:  Option<[u8; 32]>,
    pub receptor_prim: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IncomingPayload {
    pub source_space: [u8; 32],
    pub source_doc:   [u8; 32],
    pub source_prim:  String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BacklinkPayload {
    pub source_prim:   String,
    pub receptor_doc:  [u8; 32],
    pub receptor_prim: String,
}
