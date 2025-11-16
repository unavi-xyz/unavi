use serde::{Deserialize, Serialize};

use crate::WP_PREFIX;

const SCHEMA_PREFIX: &str = constcat::concat!(WP_PREFIX, "schemas/");

pub const CONNECT_URL_SCHEMA: &str = constcat::concat!(SCHEMA_PREFIX, "connect-url.json");
pub const SPACE_SCHEMA: &str = constcat::concat!(SCHEMA_PREFIX, "space.json");

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub max_players: usize,
    pub num_players: usize,
}
