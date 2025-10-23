use crate::WP_PREFIX;

const PROTOCOL_PREFIX: &str = constcat::concat!(WP_PREFIX, "protocols/");

pub const HOME_WORLD_PROTOCOL: &str = constcat::concat!(PROTOCOL_PREFIX, "home-world.json");
pub const WORLD_HOST_PROTOCOL: &str = constcat::concat!(PROTOCOL_PREFIX, "world-host.json");

pub const HOME_WORLD_DEFINITION: &[u8] =
    include_bytes!("../../../protocol/dwn/protocols/home-world.json");
pub const WORLD_HOST_DEFINITION: &[u8] =
    include_bytes!("../../../protocol/dwn/protocols/world-host.json");
