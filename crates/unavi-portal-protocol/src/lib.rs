use serde::{
    Deserialize,
    Serialize,
};

pub const INCOMING_CHANNEL: &str = "unavi:portal:incoming";
pub const BACKLINK_CHANNEL: &str = "unavi:portal:backlink";

pub const LINK_KV_KEY_PREFIX: &str = "portal:link:";

#[must_use]
pub fn link_kv_key(prim_tree_id: &str) -> String {
    format!("{LINK_KV_KEY_PREFIX}{prim_tree_id}")
}

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
